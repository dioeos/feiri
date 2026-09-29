mod handlers;
mod ipc;
mod niri;
mod services;

use std::{env::var_os, path::PathBuf, sync::Arc};

use anyhow::Context;
use tokio::sync::Mutex;
use tokio::sync::broadcast::{Receiver as BroadcastReceiver, Sender as BroadcastSender};
use tokio::sync::mpsc::{self, Receiver, Sender};
use tracing::info;
use tracing_subscriber::{EnvFilter, fmt};

use crate::handlers::{ActionHandler, QueryHandler};
use crate::niri::WindowManager;
use crate::{
    ipc::IpcServer,
    niri::{EventHandler, Listener},
    services::{MarkService, WindowService},
};

pub const FEIRI_IPC_SOCK: &str = "feiri-ipc.sock";

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenvy::dotenv().ok();
    let format = fmt::format().with_level(true).with_target(true).compact();

    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .event_format(format)
        .init();

    let niri_socket_for_listener = niri_ipc::socket::Socket::connect()
        .context("Failed to connect to niri IPC for listener")?;
    let niri_socket_for_window_manager = niri_ipc::socket::Socket::connect()
        .context("Failed to connect to niri IPC for requester")?;

    let (niri_evt_tx, mut niri_evt_rx): (Sender<niri_ipc::Event>, Receiver<niri_ipc::Event>) =
        mpsc::channel(32);

    let (daemon_evt_tx, _): (
        BroadcastSender<feiri_ipc::Event>,
        BroadcastReceiver<feiri_ipc::Event>,
    ) = tokio::sync::broadcast::channel(32);

    let niri_window_manager = Mutex::new(WindowManager::new(niri_socket_for_window_manager));

    let niri_listener = Listener::new(niri_socket_for_listener, niri_evt_tx);

    let mark_service = Arc::new(MarkService::new(niri_window_manager));
    let window_service = Arc::new(WindowService::new());

    let niri_event_handler =
        EventHandler::new(Arc::clone(&mark_service), Arc::clone(&window_service));

    let xdg_os_string =
        var_os("XDG_RUNTIME_DIR").context("XDG_RUNTIME_DIR environment variable is not set")?;

    let mut feiri_ipc_path = PathBuf::from(xdg_os_string);
    feiri_ipc_path.push(FEIRI_IPC_SOCK);

    let action_handler = ActionHandler::new(
        Arc::clone(&mark_service),
        Arc::clone(&window_service),
        daemon_evt_tx.clone(),
    );
    let query_handler = QueryHandler::new(Arc::clone(&mark_service), Arc::clone(&window_service));

    let ipc_server = Arc::new(
        IpcServer::new(feiri_ipc_path, action_handler, query_handler, daemon_evt_tx).await?,
    );
    info!("ipc server running...");

    //@TODO: Switch to old async niri stream reader
    tokio::try_join!(
        async move {
            tokio::task::spawn_blocking(move || niri_listener.run()).await??;
            Ok::<(), anyhow::Error>(())
        },
        async {
            ipc_server.run().await?;
            Ok::<(), anyhow::Error>(())
        },
        async move {
            while let Some(niri_event) = niri_evt_rx.recv().await {
                niri_event_handler.handle_event(niri_event).await;
            }
            Ok::<(), anyhow::Error>(())
        }
    )?;

    Ok(())
}
