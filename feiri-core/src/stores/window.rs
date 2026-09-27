use crate::models::{Window, WindowId};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct WindowStore {
    windows: RwLock<HashMap<WindowId, Window>>,
}

impl WindowStore {
    pub fn new() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
        }
    }

    pub async fn upsert_window(&self, window_id: WindowId, window: Window) {
        let mut rw_guard = self.windows.write().await;
        rw_guard.insert(window_id, window);
    }

    pub async fn remove_window(&self, window_id: WindowId) {
        let mut rw_guard = self.windows.write().await;
        rw_guard.remove(&window_id);
    }

    pub async fn get_window(&self, window_id: WindowId) -> Option<Window> {
        let rw_guard = self.windows.read().await;
        rw_guard.get(&window_id).cloned()
    }
}
