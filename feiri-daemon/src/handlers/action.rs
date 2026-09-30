use std::sync::Arc;

use super::Error;
use crate::services::{MarkService, WindowService};
use feiri_core::models::{Mark, WindowId};
use feiri_ipc::Action;
use tracing::debug;

pub struct ActionHandler {
    mark_service: Arc<MarkService>,
    window_service: Arc<WindowService>,
    event_sender: tokio::sync::broadcast::Sender<feiri_ipc::Event>,
}

impl ActionHandler {
    pub fn new(
        mark_service: Arc<MarkService>,
        window_service: Arc<WindowService>,
        event_sender: tokio::sync::broadcast::Sender<feiri_ipc::Event>,
    ) -> Self {
        Self {
            mark_service,
            window_service,
            event_sender,
        }
    }

    pub async fn handle_action_request(&self, action: Action) -> Result<(), Error> {
        match action {
            Action::MarkWindow { slot } => {
                self.mark_service.mark_focused_window(slot).await?;
                let marks = self.build_marks().await;
                self.emit_event(feiri_ipc::Event::MarksChanged { marks });
            }
            Action::MarkRequestedWindow { slot, id } => {
                self.mark_service.mark_window(slot, WindowId(id)).await?;
                let marks = self.build_marks().await;
                self.emit_event(feiri_ipc::Event::MarksChanged { marks });
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

    fn emit_event(&self, event: feiri_ipc::Event) {
        //a send operation can only fail if there are no active receivers
        //
        //https://docs.rs/tokio/latest/tokio/sync/broadcast/error/struct.SendError.html
        //
        //The daemon does not care if there are no receivers since the main receiver (UI) is optional
        let _ = self.event_sender.send(event);
    }

    async fn build_marks(&self) -> Vec<Mark> {
        let slot_to_window_id: Vec<(usize, WindowId)> = self.mark_service.list_marks().await;
        let mut marks: Vec<Mark> = Vec::with_capacity(slot_to_window_id.len());

        for (slot, window_id) in slot_to_window_id {
            if let Some(window) = self.window_service.find_window(window_id).await {
                marks.push(Mark { slot, window });
            }
        }

        marks
    }
}
