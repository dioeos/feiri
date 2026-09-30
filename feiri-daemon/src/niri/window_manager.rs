use feiri_core::models::{Window, WindowId};
use niri_ipc::{Action, Request, Response, socket::Socket};
use tokio::sync::Mutex;

use super::{error::CompositorError, util};
use crate::niri::conversions::IntoFeiriWindow;
pub struct WindowManager {
    socket: Mutex<Socket>,
}

impl WindowManager {
    pub fn new(socket: Mutex<Socket>) -> Self {
        Self { socket }
    }

    pub async fn get_focused_window(&self) -> Result<Option<Window>, CompositorError> {
        let request = Request::FocusedWindow;
        let mut socket_guard = self.socket.lock().await;
        let send_result = socket_guard.send(request);

        let response = util::unwrap_send_result(send_result)?;

        let niri_window = match response {
            Response::FocusedWindow(Some(window)) => Some(window),
            Response::FocusedWindow(None) => None,
            other => {
                return Err(CompositorError::UnexpectedResponse(format!("{other:?}")))?;
            }
        };

        let Some(niri_window) = niri_window else {
            return Ok(None);
        };

        Ok(Some(niri_window.into_feiri_window()))
    }

    pub async fn focus_window(&self, id: WindowId) -> Result<(), CompositorError> {
        let action = Action::FocusWindow { id: id.0 };
        let request = Request::Action(action);
        let mut socket_guard = self.socket.lock().await;
        let send_result = socket_guard.send(request);

        let response = util::unwrap_send_result(send_result)?;

        match response {
            Response::Handled => Ok(()),
            other => Err(CompositorError::UnexpectedResponse(format!("{other:?}"))),
        }
    }
}
