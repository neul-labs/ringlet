//! Async bridge between portable-pty and Tokio.
//!
//! portable-pty is synchronous, so we use spawn_blocking and channels
//! to integrate it with the async Tokio runtime.

use super::session::{SessionState, TerminalInput, TerminalOutput, TerminalSession};
use anyhow::{Context, Result};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Spawn an agent process in a PTY and bridge it to a TerminalSession.
pub async fn spawn_pty_session(
    session: Arc<TerminalSession>,
    command: &str,
    args: &[String],
    env: HashMap<String, String>,
    working_dir: &Path,
    initial_size: PtySize,
    mut input_rx: mpsc::Receiver<TerminalInput>,
) -> Result<()> {
    let command = command.to_string();
    let args = args.to_vec();
    let working_dir = working_dir.to_path_buf();
    let output_tx = session.output_sender();

    // Create PTY system
    let pty_system = native_pty_system();

    // Open a pseudo-terminal
    let pair = pty_system
        .openpty(initial_size)
        .context("Failed to open PTY")?;

    // Build command
    let mut cmd = CommandBuilder::new(&command);
    cmd.args(&args);
    cmd.cwd(&working_dir);

    // Set environment variables
    // Start with essential vars
    for key in &["PATH", "TERM", "LANG", "LC_ALL", "USER", "SHELL", "HOME"] {
        if let Ok(value) = std::env::var(key) {
            cmd.env(key, value);
        }
    }
    // Set TERM if not present
    if std::env::var("TERM").is_err() {
        cmd.env("TERM", "xterm-256color");
    }
    // Add profile-specific env vars
    for (key, value) in &env {
        cmd.env(key, value);
    }

    // Spawn the child process
    let child = pair
        .slave
        .spawn_command(cmd)
        .context("Failed to spawn command in PTY")?;

    // Get PID
    let pid = child.process_id();
    if let Some(pid) = pid {
        session.set_pid(pid).await;
        info!("PTY session {} spawned with PID {}", session.id, pid);
    }

    // Mark session as running
    session.set_state(SessionState::Running).await;

    // Get master PTY for reading/writing
    let master = pair.master;

    // Clone for the reader task
    let reader_master = master
        .try_clone_reader()
        .context("Failed to clone PTY reader")?;

    // Take the writer handle for data I/O (implements std::io::Write)
    // Note: take_writer can only be called once per master
    let writer_handle_pty = master
        .take_writer()
        .context("Failed to take PTY writer")?;

    // Keep master for resize operations (resize is still on MasterPty)
    let master_for_resize = master;

    let session_id = session.id.clone();

    // Create a channel for passing data from the blocking reader to async scrollback writer
    let (scrollback_tx, mut scrollback_rx) = mpsc::channel::<Vec<u8>>(256);
    let session_for_scrollback = session.clone();

    // Spawn async task to write to scrollback buffer
    let scrollback_handle = tokio::spawn(async move {
        while let Some(data) = scrollback_rx.recv().await {
            session_for_scrollback.append_scrollback(&data).await;
        }
    });

    // Spawn blocking reader task (PTY output -> broadcast + scrollback channel)
    let reader_handle = tokio::task::spawn_blocking(move || {
        let mut reader = reader_master;
        let mut buffer = [0u8; 4096];

        loop {
            match reader.read(&mut buffer) {
                Ok(0) => {
                    // EOF - process exited
                    debug!("PTY reader EOF for session {}", session_id);
                    break;
                }
                Ok(n) => {
                    let data = buffer[..n].to_vec();
                    // Send to scrollback channel (best effort, don't block)
                    let _ = scrollback_tx.try_send(data.clone());
                    // Broadcast to connected clients (ignore errors if no receivers)
                    let _ = output_tx.send(TerminalOutput::Data(data));
                }
                Err(e) => {
                    // Check if it's a "would block" or similar transient error
                    if e.kind() == std::io::ErrorKind::Interrupted {
                        continue;
                    }
                    error!("PTY read error for session {}: {}", session_id, e);
                    break;
                }
            }
        }
    });

    let session_id_writer = session.id.clone();

    // Use Arc<Mutex> for the writer and master to share between async and blocking contexts
    let writer_pty = Arc::new(std::sync::Mutex::new(writer_handle_pty));
    let master_pty = Arc::new(std::sync::Mutex::new(master_for_resize));

    // Spawn writer task (input channel -> PTY)
    let writer_handle = tokio::spawn(async move {
        while let Some(input) = input_rx.recv().await {
            match input {
                TerminalInput::Data(data) => {
                    let writer_clone = writer_pty.clone();
                    let session_id = session_id_writer.clone();
                    let result: Result<(), std::io::Error> = tokio::task::spawn_blocking(move || {
                        let mut writer = writer_clone.lock().unwrap();
                        writer.write_all(&data)?;
                        writer.flush()
                    })
                    .await
                    .unwrap_or_else(|e| Err(std::io::Error::other(e.to_string())));
                    if let Err(e) = result {
                        error!("PTY write error for session {}: {}", session_id, e);
                        break;
                    }
                }
                TerminalInput::Resize { cols, rows } => {
                    let master_clone = master_pty.clone();
                    let session_id = session_id_writer.clone();
                    let size = PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    };
                    let result: Result<(), anyhow::Error> = tokio::task::spawn_blocking(move || {
                        let master = master_clone.lock().unwrap();
                        master.resize(size)
                    })
                    .await
                    .unwrap_or_else(|e| Err(anyhow::anyhow!(e.to_string())));
                    if let Err(e) = result {
                        warn!("PTY resize error for session {}: {}", session_id, e);
                    } else {
                        debug!("PTY resized to {}x{} for session {}", cols, rows, session_id_writer);
                    }
                }
                TerminalInput::Signal(sig) => {
                    // Note: portable-pty doesn't have direct signal support
                    // For now, we can write control characters for common signals
                    let ctrl_char = match sig {
                        2 => Some(b'\x03'),  // SIGINT -> Ctrl+C
                        3 => Some(b'\x1c'),  // SIGQUIT -> Ctrl+\
                        28 => None,          // SIGWINCH handled by resize
                        _ => {
                            warn!("Unsupported signal {} for session {}", sig, session_id_writer);
                            None
                        }
                    };
                    if let Some(c) = ctrl_char {
                        let writer_clone = writer_pty.clone();
                        let session_id = session_id_writer.clone();
                        let result: Result<(), std::io::Error> = tokio::task::spawn_blocking(move || {
                            let mut writer = writer_clone.lock().unwrap();
                            writer.write_all(&[c])
                        })
                        .await
                        .unwrap_or_else(|e| Err(std::io::Error::other(e.to_string())));
                        if let Err(e) = result {
                            error!("Failed to send signal {} to session {}: {}", sig, session_id, e);
                        }
                    }
                }
            }
        }
    });

    // Wait for child process to exit
    let mut child = child;
    let exit_status = tokio::task::spawn_blocking(move || child.wait())
        .await
        .context("Failed to wait for child process")?;

    // Log first, then consume the exit_status
    info!(
        "PTY session {} terminated with exit status: {:?}",
        session.id, exit_status
    );

    let exit_code = exit_status.ok().and_then(|s| {
        if s.success() {
            Some(0)
        } else {
            // portable-pty ExitStatus doesn't expose code directly on all platforms
            // Try to get it if available
            None
        }
    });

    // Mark session as terminated
    session
        .set_state(SessionState::Terminated { exit_code })
        .await;

    // Clean up tasks
    reader_handle.abort();
    writer_handle.abort();
    scrollback_handle.abort();

    Ok(())
}
