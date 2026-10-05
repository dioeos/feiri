use feiri_ipc::{
    Action,
    Command::{self},
    Event, Reply, Request, Response,
    socket::Socket,
};
use slint::{Model, SharedString, VecModel};

use crate::{
    AppWindow, MarkRowItem, config,
    data::{MarkData, MarksState},
};

use super::error::Error;

//@NOTE: Call only on the Slint UI thread: Image is not Send.
fn to_ui_row(row: MarkData) -> MarkRowItem {
    MarkRowItem {
        slot: row.slot.into(),
        title: row.title.into(),
        icon: row
            .app_id
            .as_deref()
            .and_then(config::icon_for_app_id)
            .unwrap_or_default(),
    }
}

pub async fn handle_event(
    event: feiri_ipc::Event,
    marks_state: MarksState,
    weak_ui: slint::Weak<AppWindow>,
) -> Result<(), Error> {
    match event {
        Event::MarksChanged { marks } => {
            let rows: Vec<MarkData> = marks
                .into_iter()
                .map(|mark| {
                    let app_id = mark.window.app_id;
                    MarkData {
                        slot: mark.slot.to_string(),
                        title: mark
                            .window
                            .title
                            .or_else(|| app_id.clone())
                            .unwrap_or_else(|| "Untitled".into()),
                        app_id,
                    }
                })
                .collect();

            *marks_state.lock().await = rows.clone();

            weak_ui.upgrade_in_event_loop(move |ui| {
                let ui_rows: Vec<MarkRowItem> = rows.into_iter().map(to_ui_row).collect();

                let model = ui.get_marks();

                let model = model
                    .as_any()
                    .downcast_ref::<VecModel<MarkRowItem>>()
                    .expect("marks backed by Vec<T>");
                model.set_vec(ui_rows);
            })?;
        }
    }
    Ok(())
}

pub async fn handle_delete(
    slot: SharedString,
    weak_ui: slint::Weak<AppWindow>,
) -> Result<(), Error> {
    let mut socket = Socket::connect().await?;
    let slot_u8 = slot.clone().parse::<u8>().unwrap();
    let action = Action::DeleteMark { slot: slot_u8 };

    let reply: Reply = socket
        .send(Request::Operation(Command::Action(action)))
        .await?;

    let response = match reply {
        Ok(response) => response,
        Err(message) => return Err(Error::FeiriErrorRequest(message.to_string())),
    };

    if !matches!(response, Response::Handled) {
        return Err(Error::UnexpectedIpcResponse {
            expected: Response::Handled,
            received: response,
        });
    }

    weak_ui
        .upgrade_in_event_loop(move |ui| {
            let model = ui.get_marks();

            for i in 0..model.row_count() {
                if let Some(mark) = model.row_data(i)
                    && mark.slot == slot
                {
                    let model = model
                        .as_any()
                        .downcast_ref::<VecModel<MarkRowItem>>()
                        .expect("marks backed by Vec<T>");

                    model.remove(i);
                }
            }
        })
        .unwrap();

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
            .collect::<Vec<MarkData>>()
    };

    let Some(ui) = weak_ui.upgrade() else {
        return Err(Error::UIDropped);
    };

    let model = ui.get_marks();
    let model = model
        .as_any()
        .downcast_ref::<VecModel<MarkRowItem>>()
        .expect("marks backed by Vec<T>");
    model.set_vec(filtered_rows.into_iter().map(to_ui_row).collect::<Vec<_>>());
    Ok(())
}

pub async fn handle_focus(slot: SharedString) -> Result<(), Error> {
    let slot = slot
        .parse::<u8>()
        .map_err(|_| Error::FailedToConvertIndexToSlot)?;
    let mut socket = Socket::connect().await?;

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
