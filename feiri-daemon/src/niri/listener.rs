use niri_ipc::{Request, Response, socket::Socket};
use tokio::sync::mpsc::Sender;
use tracing::debug;

use super::{
    error::{CompositorError, Error},
    util,
};

pub struct Listener {
    socket: Socket,
    niri_evt_tx: Sender<niri_ipc::Event>,
}

impl Listener {
    pub fn new(socket: Socket, niri_evt_tx: Sender<niri_ipc::Event>) -> Self {
        Self {
            socket,
            niri_evt_tx,
        }
    }

    pub fn run(mut self) -> Result<(), Error> {
        let request = Request::EventStream;
        let send_result = self.socket.send(request);

        let response = util::unwrap_send_result(send_result)?;

        if !matches!(response, Response::Handled) {
            return Err(CompositorError::UnexpectedResponse(format!("{response:?}")))?;
        }

        let mut read_event = self.socket.read_events();
        while let Ok(event) = read_event() {
            debug!("received niri event: {event:?}");
            self.niri_evt_tx
                .blocking_send(event)
                .map_err(|_| Error::EventChannelClosed)?;
        }
        Ok(())
    }
}
