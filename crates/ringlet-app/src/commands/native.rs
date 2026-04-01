use crate::error::AppError;
use tauri_plugin_dialog::DialogExt;

/// Open a native OS directory picker dialog.
#[tauri::command]
pub async fn pick_directory(app: tauri::AppHandle) -> Result<Option<String>, AppError> {
    let (tx, rx) = tokio::sync::oneshot::channel();

    app.dialog().file().pick_folder(move |path| {
        let _ = tx.send(path.map(|p| p.to_string()));
    });

    let result = rx.await.map_err(|_| {
        AppError::Other("Dialog was cancelled".to_string())
    })?;

    Ok(result)
}
