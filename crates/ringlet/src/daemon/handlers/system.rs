//! System-level handlers used by the HTTP layer.

use crate::daemon::server::ServerState;

pub async fn shutdown(state: &ServerState) {
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
}
