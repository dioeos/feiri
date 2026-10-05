mod callbacks;
mod error;

use error::Error;

use std::{rc::Rc, sync::Arc};

use feiri_ipc::{Reply, Request, Response, socket::Socket};
use slint::VecModel;
use tokio::{runtime, sync::Mutex};
use tokio_stream::StreamExt;
use tracing::{error, info};
use tracing_subscriber::{EnvFilter, fmt};

slint::include_modules!();

fn main() -> Result<(), Error> {
    dotenvy::dotenv().ok();
    let format = fmt::format().with_level(true).with_target(true).compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .event_format(format)
        .init();

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(1)
        .enable_io()
        .build()
        .unwrap();

    let _rt_guard = rt.enter();
    info!("entered tokio runtime");

    let ui = AppWindow::new()?;

    //@NOTE: Arc<Mutex<T>> is preferred over `Rc` due to `marks_state` being captured by a closure
    //       passed to `slint_invoke_from_event_loop` from a `tokio::spawn` task. `Rc` does not have
    //       `Send`, making it incorrect choice to wrap the state. The `all_marks_state` role is to
    //       share state between the tokio background worker thread and the main Slint event thread.
    let all_marks_state = Arc::new(Mutex::new(Vec::<MarkRowItem>::new()));
    let all_marks_state_for_event_stream = Arc::clone(&all_marks_state);

    let marks_model = Rc::new(VecModel::<MarkRowItem>::default());
    ui.set_marks(marks_model.into());

    let weak_ui = ui.as_weak();
    let weak_ui_for_event_stream = weak_ui.clone();
    tokio::spawn(async move {
        let mut socket = Socket::connect().await?;
        //socket can return errors defined in `socket::Error`
        //unwraps the outermost error representing failed communication with Feiri
        let reply: Reply = socket.send(Request::EventStream).await?;

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

        //returns impl Stream<Item = Result<Event, Error>>, giving a stream to iterate
        let mut events = socket.read_events().await;

        while let Some(event_result) = events.next().await {
            match event_result {
                Ok(event) => {
                    callbacks::handle_event(
                        event,
                        Arc::clone(&all_marks_state_for_event_stream),
                        weak_ui_for_event_stream.clone(),
                    )
                    .await
                    .map_err(|err| Error::FailedToHandleEvent(err.to_string()))?;
                }
                Err(err) => {
                    error!("failed to read event: {err:?}");
                    Err(Error::FailedToReadEvent(err.to_string()))?;
                }
            }
        }
        Ok::<(), Error>(())
    });

    let weak_ui_tmp = weak_ui.clone();
    ui.on_search_requested(move |search_input| {
        let query = search_input.to_string().to_lowercase();
        let all_marks_wrapper_for_search = Arc::clone(&all_marks_state);
        let weak_ui_for_search = weak_ui_tmp.clone();
        let search_result = slint::spawn_local(async_compat::Compat::new(async move {
            if let Err(err) =
                callbacks::handle_search(query, all_marks_wrapper_for_search, weak_ui_for_search)
                    .await
            {
                error!("failed to filter marks: {err:?}");
            }
        }))
        .map_err(Error::EventLoopError);

        if let Err(err) = search_result {
            error!("failed to spawn search: {err:?}");
        }
    });

    let weak_ui_for_focus = weak_ui.clone();
    ui.on_focus_requested(move |slot| {
        let weak_ui = weak_ui_for_focus.clone();
        let focus_result = slint::spawn_local(async_compat::Compat::new(async move {
            if let Err(err) = callbacks::handle_focus(slot).await {
                error!("failed to focus mark: {err:?}");
                return;
            }

            let Some(ui) = weak_ui.upgrade() else {
                error!("failed to find window in focus request");
                return;
            };

            if let Err(err) = ui.hide() {
                error!("failed to hide window in focus request: {err:?}");
            }
        }))
        .map_err(Error::EventLoopError);

        if let Err(err) = focus_result {
            error!("failed to spawn focus: {err:?}");
        }
    });

    let weak_ui_for_delete = weak_ui.clone();
    ui.on_delete_requested(move |slot| {
        let weak_ui = weak_ui_for_delete.clone();
        tokio::spawn(async move {
            callbacks::handle_delete(slot, weak_ui).await?;

            Ok::<(), Error>(())
        });
    });

    ui.run()?;
    Ok(())
}
