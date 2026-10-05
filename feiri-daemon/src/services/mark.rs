use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::debug;

use feiri_core::{models::WindowId, stores::MarkStore};

use super::Error;
use crate::WindowManager;

pub struct MarkService {
    mark_store: Arc<MarkStore>,
    window_manager: WindowManager,
    last_focused_slot: Mutex<Option<usize>>,
}

#[allow(dead_code)]
impl MarkService {
    pub fn new(window_manager: WindowManager) -> Self {
        Self {
            mark_store: Arc::new(MarkStore::default()),
            window_manager,
            last_focused_slot: Mutex::new(None),
        }
    }

    pub async fn list_marks(&self) -> Vec<(usize, WindowId)> {
        self.mark_store.all_marks().await
    }

    pub async fn delete_mark(&self, slot: u8)  {
        match self.mark_store.remove_mark(slot).await {
            Some(_) => {}
            None => {
                debug!("Cannot delete mark. No mark in given slot");
            }
        }
    }

    pub async fn mark_focused_window(&self, slot: u8) -> Result<(), Error> {
        let Some(focused_window) = self.window_manager.get_focused_window().await? else {
            debug!("Cannot mark window. No current focused window");
            return Ok(());
        };

        let window_id = focused_window.id;

        self.mark_store.insert_mark(slot, focused_window.id).await;
        self.set_last_focused_slot(usize::from(slot - 1)).await;
        debug!(window_id = window_id.0, mark = slot, "mark focused window");
        Ok(())
    }

    pub async fn mark_window_next(&self) -> Result<(), Error> {
        let Some(focused_window) = self.window_manager.get_focused_window().await? else {
            debug!("Cannot mark window. No current focused window");
            return Ok(());
        };

        let window_id = focused_window.id;
        let slot = match self.mark_store.next_available_slot().await {
            Some(val) => val,
            None => return Err(Error::NoAvailableSlot),
        };

        self.mark_store.insert_mark(slot, focused_window.id).await;
        self.set_last_focused_slot(usize::from(slot - 1)).await;
        debug!(window_id = window_id.0, mark = slot, "mark focused window");
        Ok(())
    }

    pub async fn mark_window(&self, slot: u8, window_id: WindowId) -> Result<(), Error> {
        self.mark_store.insert_mark(slot, window_id).await;
        debug!(window_id = window_id.0, mark = slot, "mark window");
        Ok(())
    }

    pub async fn focus_marked_window(&self, slot: u8) -> Result<(), Error> {
        let Some(window_id) = self.mark_store.get_mark(slot).await else {
            debug!(mark = slot, "no window marked");
            return Ok(());
        };

        self.window_manager.focus_window(window_id).await?;
        self.set_last_focused_slot(usize::from(slot - 1)).await;
        Ok(())
    }

    pub async fn focus_next_marked_window(&self) -> Result<(), Error> {
        let last_slot = *self.last_focused_slot.lock().await;

        let Some(slot) = last_slot else {
            debug!("no slot");
            return Ok(());
        };

        let (Some(index), Some(window_id_to_focus)) = self.mark_store.next_mark(slot).await else {
            debug!(mark = slot, "no window marked");
            return Ok(());
        };

        self.window_manager.focus_window(window_id_to_focus).await?;
        self.set_last_focused_slot(index).await;

        Ok(())
    }

    pub async fn focus_prev_marked_window(&self) -> Result<(), Error> {
        let last_slot = *self.last_focused_slot.lock().await;

        let Some(slot) = last_slot else {
            debug!("no slot");
            return Ok(());
        };

        let (Some(index), Some(window_id_to_focus)) = self.mark_store.prev_mark(slot).await else {
            debug!(mark = slot, "no window marked");
            return Ok(());
        };

        self.window_manager.focus_window(window_id_to_focus).await?;
        self.set_last_focused_slot(index).await;

        Ok(())
    }

    async fn set_last_focused_slot(&self, slot: usize) {
        *self.last_focused_slot.lock().await = Some(slot);
    }
}
