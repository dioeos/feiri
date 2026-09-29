use anyhow::Context;
use clap::Parser;
use feiri_cli::{Cli, Msg, Sub};
use feiri_ipc::Request;
use tracing::debug;
use tracing_subscriber::{EnvFilter, fmt};

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

    let cli = Cli::parse();

    if let Some(Sub::Msg { msg }) = cli.subcommand {
        let mut socket = feiri_ipc::socket::Socket::connect()
            .await
            .context("Failed to connect to Feiri IPC")?;

        match msg {
            Msg::Action { action } => {
                socket
                    .send(Request::Action(action.clone()))
                    .await
                    .context("Failed to send Feiri IPC action")?;
                debug!(?action, "sent msg");
            }
            Msg::Query { query } => {
                socket
                    .send(Request::Query(query.clone()))
                    .await
                    .context("Failed to send Feiri IPC query")?;
                debug!(?query, "sent msg");
            }
        }
    }

    Ok(())
}
