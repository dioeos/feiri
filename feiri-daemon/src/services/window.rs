use feiri_core::{
    models::{Window, WindowId},
    stores::WindowStore
};

use tracing::debug;

pub struct WindowService {
    window_store: WindowStore,
}

impl WindowService {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            window_store: WindowStore::new(),
        }
    }
    pub async fn upsert_window(&self, window: Window) {
        debug!(
            window_id = ?window.id,
            window = ?window,
            "inserting window"
        );
        self.window_store.upsert_window(window.id, window).await;
    }

    pub async fn remove_window(&self, window_id: WindowId) {
        debug!(
            window_id = ?window_id,
            "removing window"
        );
        self.window_store.remove_window(window_id).await;
    }
}
