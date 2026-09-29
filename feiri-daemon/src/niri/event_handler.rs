use std::sync::Arc;

use feiri_core::models::WindowId;
use tracing::{debug, trace};

use crate::{
    niri::conversions::IntoFeiriWindow,
    services::{MarkService, WindowService},
};

#[allow(dead_code)]
pub struct EventHandler {
    mark_service: Arc<MarkService>,
    window_service: Arc<WindowService>,
}

impl EventHandler {
    pub fn new(mark_service: Arc<MarkService>, window_service: Arc<WindowService>) -> Self {
        Self {
            mark_service,
            window_service,
        }
    }

    //@NOTE: Currently infallible, only fails during panic situations (will later handle and
    //       propogate)
    pub async fn handle_event(&self, event: niri_ipc::Event) {
        match event {
            niri_ipc::Event::WindowOpenedOrChanged { window } => {
                debug!(window = ?window, "window opened or changed");
                let domain_window = window.into_feiri_window();
                self.window_service.upsert_window(domain_window).await;
            }
            niri_ipc::Event::WindowClosed { id } => {
                debug!(window_id = id, "window closed");
                self.window_service.remove_window(WindowId(id)).await;
            }
            niri_ipc::Event::WindowsChanged { windows } => {
                debug!(windows_count = windows.len(), "windows changed");
                for w in windows {
                    let domain_window = w.into_feiri_window();
                    self.window_service.upsert_window(domain_window).await;
                }
            }
            _ => {
                trace!(?event, "ignoring unsupported niri event");
            }
        }
    }
}
