use std::sync::Arc;

use super::Error;
use crate::services::MarkService;
use feiri_core::models::WindowId;
use feiri_ipc::Action;
use tracing::debug;

pub struct ActionHandler {
    mark_service: Arc<MarkService>,
}

impl ActionHandler {
    pub fn new(mark_service: Arc<MarkService>) -> Self {
        Self { mark_service }
    }

    pub async fn handle_action_request(&self, action: Action) -> Result<(), Error> {
        match action {
            Action::MarkWindow { slot } => {
                self.mark_service.mark_focused_window(slot).await?;
            }
            Action::MarkRequestedWindow { slot, id } => {
                self.mark_service.mark_window(slot, WindowId(id)).await?;
            }
            Action::FocusMark { slot } => {
                self.mark_service.focus_marked_window(slot).await?;
            }
            Action::NextMark => {
                self.mark_service.focus_next_marked_window().await?;
            }
            Action::PrevMark => {
                self.mark_service.focus_prev_marked_window().await?;
            }
        }
        debug!(action = ?action, "successfully handled");
        Ok(())
    }
}
