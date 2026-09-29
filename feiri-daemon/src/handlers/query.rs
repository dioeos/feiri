use std::sync::Arc;

use crate::services::{MarkService, WindowService};
use feiri_core::models::{Mark, WindowId};
use feiri_ipc::{Query, QueryResponse};

use super::Error;

pub(crate) struct QueryHandler {
    mark_service: Arc<MarkService>,
    window_service: Arc<WindowService>,
}

impl QueryHandler {
    pub(crate) fn new(mark_service: Arc<MarkService>, window_service: Arc<WindowService>) -> Self {
        Self {
            mark_service,
            window_service,
        }
    }

    pub(crate) async fn handle_query_request(
        &self,
        request: Query,
    ) -> Result<QueryResponse, Error> {
        match request {
            Query::ListMarks => {
                let slot_to_window_id: Vec<(usize, WindowId)> =
                    self.mark_service.list_marks().await;

                let mut marks: Vec<Mark> = Vec::with_capacity(slot_to_window_id.len());

                for (slot, window_id) in slot_to_window_id {
                    if let Some(window) = self.window_service.find_window(window_id).await {
                        marks.push(Mark { slot, window });
                    }
                }

                Ok(QueryResponse::Marks(marks))
            }
        }
    }
}
