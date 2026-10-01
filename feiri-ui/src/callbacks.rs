use std::sync::Arc;

use feiri_ipc::{Action, Command, Event, Request, Response, socket::Socket};
use slint::{Model, SharedString, VecModel};
use tokio::sync::Mutex;

use crate::{AppWindow, MarkRowItem};

use super::error::Error;

type MarksState = Arc<Mutex<Vec<MarkRowItem>>>;

pub async fn handle_event(
    event: feiri_ipc::Event,
    marks_state: MarksState,
    weak_ui: slint::Weak<AppWindow>,
) -> Result<(), Error> {
    match event {
        Event::MarksChanged { marks } => {
            let rows: Vec<MarkRowItem> = marks
                .into_iter()
                .map(|mark| MarkRowItem {
                    slot: SharedString::from(mark.slot.to_string()),
                    title: SharedString::from(
                        mark.window
                            .title
                            .or(mark.window.app_id)
                            .unwrap_or_else(|| "Untitled".into()),
                    ),
                })
                .collect();

            *marks_state.lock().await = rows.clone();

            weak_ui
                .upgrade_in_event_loop(move |ui| {
                    let model = ui.get_marks();

                    let model = model
                        .as_any()
                        .downcast_ref::<VecModel<MarkRowItem>>()
                        .expect("marks backed by Vec<T>");
                    model.set_vec(rows);
                })
                .unwrap();
        }
    }
    Ok(())
}

pub async fn handle_search(
    query: String,
    marks_state: MarksState,
    weak_ui: slint::Weak<AppWindow>,
) -> Result<(), Error> {
    let filtered_rows = {
        let wrapper_guard = marks_state.lock().await;

        wrapper_guard
            .iter()
            .filter(|row| row.title.to_lowercase().contains(&query))
            .cloned()
            .collect::<Vec<MarkRowItem>>()
    };

    let Some(ui) = weak_ui.upgrade() else {
        return Err(Error::UIDropped);
    };

    let model = ui.get_marks();
    let model = model
        .as_any()
        .downcast_ref::<VecModel<MarkRowItem>>()
        .expect("marks backed by Vec<T>");
    model.set_vec(filtered_rows);
    Ok(())
}

pub async fn handle_focus(current_index: i32) -> Result<(), Error> {
    let mut socket = Socket::connect().await?;
    let slot = match u8::try_from(current_index) {
        Ok(val) => val,
        Err(_) => return Err(Error::FailedToConvertIndexToSlot),
    };

    let action = Action::FocusMark { slot };

    let reply = socket
        .send(Request::Operation(Command::Action(action)))
        .await?;

    let response = match reply {
        Ok(response) => response,
        Err(msg) => return Err(Error::FeiriErrorRequest(msg.to_string())),
    };

    if !matches!(response, Response::Handled) {
        return Err(Error::UnexpectedIpcResponse {
            expected: Response::Handled,
            received: response,
        });
    }
    Ok::<(), Error>(())
}
