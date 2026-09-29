use std::io;

use super::error::CompositorError;
use niri_ipc::{Reply, Response};
use tracing::error;

pub(crate) fn unwrap_send_result(
    send_result: io::Result<Reply>,
) -> Result<Response, CompositorError> {
    let response = match send_result {
        Ok(response) => response,
        Err(err) => {
            error!("failed to sent request to niri: {err:#}");
            return Err(CompositorError::FailedRequestCommunication(err.to_string()));
        }
    };

    let response = match response {
        Ok(response) => response,
        Err(err) => {
            error!("error response from niri: {err:#}");
            return Err(CompositorError::ErrorResponse(err.to_string()));
        }
    };

    Ok(response)
}
