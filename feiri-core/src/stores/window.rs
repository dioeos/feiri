use crate::models::{Window, WindowId};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct WindowStore {
    windows: RwLock<HashMap<WindowId, Window>>,
}

impl Default for WindowStore {
    fn default() -> Self {
        Self {
            windows: RwLock::new(HashMap::new()),
        }
    }
}

impl WindowStore {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_window() -> Window {
        Window {
            id: WindowId(1),
            title: Some("YouTube - Mozilla FireFox".into()),
            app_id: Some("org.mozilla.firefox".into()),
        }
    }

    #[tokio::test]
    async fn updates_when_existing() {
        let store = WindowStore::default();
        let window = base_window();
        store.upsert_window(window.id, window).await;
        let mut updated_window = base_window();
        updated_window.title = Some("README.md - Ghostty".into());
        updated_window.app_id = Some("com.mitchellh.ghostty".into());
        store
            .upsert_window(updated_window.id, updated_window.clone())
            .await;

        let stored_window = store.get_window(updated_window.id).await.unwrap();
        assert_eq!(stored_window.id, updated_window.id);
        assert_eq!(stored_window.title, updated_window.title);
        assert_eq!(stored_window.app_id, updated_window.app_id);
    }

    #[tokio::test]
    async fn inserts_when_does_not_exist() {
        let store = WindowStore::default();
        let window = base_window();
        assert!(store.get_window(window.id).await.is_none());

        store.upsert_window(window.id, window.clone()).await;

        let stored_window = store.get_window(window.id).await.unwrap();
        assert_eq!(stored_window.id, window.id);
        assert_eq!(stored_window.title, window.title);
        assert_eq!(stored_window.app_id, window.app_id);
    }

    #[tokio::test]
    async fn removes_existing_window() {
        let store = WindowStore::default();
        let window = base_window();
        let window_id = window.id;
        store.upsert_window(window_id, window).await;
        assert!(store.get_window(window_id).await.is_some());

        store.remove_window(window_id).await;

        assert!(store.get_window(window_id).await.is_none());
    }

    #[tokio::test]
    async fn get_returns_none_when_does_not_exist() {
        let store = WindowStore::default();

        assert!(store.get_window(WindowId(1)).await.is_none());
    }
}
