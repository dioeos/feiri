use crate::models::WindowId;
use tokio::sync::RwLock;

pub struct MarkStore {
    marks: RwLock<[Option<WindowId>; 9]>,
}

impl Default for MarkStore {
    fn default() -> Self {
        Self {
            marks: RwLock::new([None; 9]),
        }
    }
}

impl MarkStore {
    pub async fn all_marks(&self) -> Vec<(usize, WindowId)> {
        self.marks
            .read()
            .await
            .iter()
            .enumerate()
            .filter_map(|(slot, id)| id.as_ref().map(|id| (slot + 1, *id)))
            .collect()
    }

    pub async fn insert_mark(&self, slot: u8, id: WindowId) {
        let index = usize::from(slot - 1);
        let mut rw_guard = self.marks.write().await;

        for mark in rw_guard.iter_mut() {
            if *mark == Some(id) {
                *mark = None;
            }
        }

        rw_guard[index] = Some(id);
    }

    //@TODO: Implement a `slot_to_index` function that asserts only positions 1-9,
    //       making the mark store completely infallible, as right now out of bounds
    //       access will cause the program to panic
    pub async fn get_mark(&self, slot: u8) -> Option<WindowId> {
        let index = usize::from(slot - 1);
        let rw_guard = self.marks.read().await;
        rw_guard[index]
    }

    pub async fn next_mark(&self, current_slot: usize) -> (Option<usize>, Option<WindowId>) {
        let rw_guard = self.marks.read().await;
        for step in 1..=rw_guard.len() {
            let index = (current_slot + step) % rw_guard.len();
            if let Some(window_id) = rw_guard[index] {
                return (Some(index), Some(window_id));
            }
        }

        (None, None)
    }

    pub async fn prev_mark(&self, current_slot: usize) -> (Option<usize>, Option<WindowId>) {
        let rw_guard = self.marks.read().await;
        for step in 1..=rw_guard.len() {
            let index = (current_slot + rw_guard.len() - step) % rw_guard.len();
            if let Some(window_id) = rw_guard[index] {
                return (Some(index), Some(window_id));
            }
        }

        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn inserts_and_gets_mark() {
        let store = MarkStore::default();
        let id = WindowId(42);

        store.insert_mark(1, id).await;

        assert_eq!(store.get_mark(1).await, Some(id));
    }

    #[tokio::test]
    async fn returns_none_for_empty_requested_slot() {
        let store = MarkStore::default();

        assert_eq!(store.get_mark(1).await, None);
    }

    #[tokio::test]
    async fn moves_existing_mark_to_requested() {
        let store = MarkStore::default();
        let id = WindowId(67);
        store.insert_mark(1, id).await;

        assert_eq!(store.get_mark(1).await, Some(id));
        store.insert_mark(2, id).await;
        assert_eq!(store.get_mark(1).await, None);
        assert_eq!(store.get_mark(2).await, Some(id));
    }

    #[tokio::test]
    async fn next_mark_wraps_when_index_overflows() {
        let store = MarkStore::default();
        let id = WindowId(67);
        store.insert_mark(1, id).await;

        let result = store.next_mark(8).await;
        assert_eq!(result, (Some(0), Some(id)));
    }

    #[tokio::test]
    async fn prev_mark_wraps_when_index_underflows() {
        let store = MarkStore::default();
        let id = WindowId(67);
        store.insert_mark(9, id).await;

        let result = store.prev_mark(0).await;
        assert_eq!(result, (Some(8), Some(id)));
    }
}
