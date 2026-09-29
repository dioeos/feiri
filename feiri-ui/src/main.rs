#![allow(dead_code, unused_variables)]
use std::{rc::Rc, sync::Arc};

use slint::VecModel;
use tokio::{runtime, sync::Mutex};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
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
    let all_marks_state_for_initial_load = Arc::clone(&all_marks_state);

    let marks_model = Rc::new(VecModel::<MarkRowItem>::default());
    ui.set_marks(marks_model.into());

    let weak_ui = ui.as_weak();
    // tokio::spawn(async move {
    // });

    Ok(())
}
