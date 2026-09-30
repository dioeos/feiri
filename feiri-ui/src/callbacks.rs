use std::sync::Arc;

use feiri_ipc::Event;
use slint::{Model, SharedString, VecModel};
use tokio::sync::Mutex;

use crate::{AppWindow, MarkRowItem};

use super::error::Error;

pub async fn handle_event(
    event: feiri_ipc::Event,
    marks_state: Arc<Mutex<Vec<MarkRowItem>>>,
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
