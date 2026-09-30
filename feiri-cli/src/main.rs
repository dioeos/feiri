use anyhow::{Context, bail};
use clap::Parser;
use feiri_cli::{Cli, Msg, Sub};
use feiri_ipc::{Command, Request, Response};
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
                let reply = socket
                    .send(Request::Operation(Command::Action(action.clone())))
                    .await
                    .context("Failed to send Feiri IPC action")?;

                let response = match reply {
                    Ok(response) => response,
                    Err(err) => bail!("Errror handling request: {err:?}")
                };

                if !matches!(response, Response::Handled) {
                    println!("Failed to perform action. Unexpected response: {response:?}");
                }
            }
            Msg::Query { query } => {
                let reply = socket
                    .send(Request::Operation(Command::Query(query.clone())))
                    .await
                    .context("Failed to send Feiri IPC query")?;

                let response = match reply {
                    Ok(response) => response,
                    Err(err) => bail!("Error handling request: {err:?}")
                };

                println!("{response:?}");
                debug!(?query, "sent msg");
            }
        }
    }

    Ok(())
}
