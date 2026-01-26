//! HTTP/WebSocket server for the web UI.
//!
//! This module provides an HTTP API that mirrors the NNG IPC protocol,
//! allowing web-based clients to interact with the daemon.

pub mod assets;
pub mod error;
pub mod routes;
pub mod server;
pub mod terminal_ws;
pub mod websocket;

pub use server::run_http_server;
