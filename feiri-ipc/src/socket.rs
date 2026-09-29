use std::{env::var_os, io, path::PathBuf};

use futures_core::Stream;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixStream,
};
use tokio_stream::{StreamExt, wrappers::LinesStream};

use crate::{Event, Reply, Request};

pub const FEIRI_IPC_SOCK: &str = "feiri-ipc.sock";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("XDG_RUNTIME_DIR is not set")]
    MissingRuntimeDir,

    #[error(transparent)]
    Io(#[from] io::Error),

    #[error(transparent)]
    Serde(#[from] serde_json::Error),
}

pub struct Socket {
    stream: BufReader<UnixStream>,
}

impl Socket {
    pub async fn connect() -> Result<Self, Error> {
        let xdg_os_string = var_os("XDG_RUNTIME_DIR").ok_or(Error::MissingRuntimeDir)?;

        Ok(Self::connect_to(PathBuf::from(xdg_os_string).join(FEIRI_IPC_SOCK)).await?)
    }

    pub async fn connect_to(path: PathBuf) -> io::Result<Self> {
        let stream = UnixStream::connect(path).await?;
        let stream = BufReader::new(stream);
        Ok(Self { stream })
    }

    pub async fn send(&mut self, request: Request) -> Result<Reply, Error> {
        let mut buf = serde_json::to_string(&request)?;
        buf.push('\n');
        self.stream.get_mut().write_all(buf.as_bytes()).await?;

        buf.clear();
        self.stream.read_line(&mut buf).await?;

        let reply = serde_json::from_str(&buf)?;
        Ok(reply)
    }

    /// Reads event stream [`Event`]s from the socket that the Feiri daemon emits.
    ///
    /// The returned function will yield until the next [`Event`] arrives, returning it.
    ///
    /// This function should only be used after requesting [`EventStream`][Request::EventStream]
    pub async fn read_events(self) -> impl Stream<Item = Result<Event, Error>> {
        let Self { stream } = self;
        let (reader, mut writer) = stream.into_inner().into_split();

        writer.shutdown().await.unwrap();

        let reader = BufReader::new(reader);
        let lines = LinesStream::new(reader.lines());

        lines.map(|line| {
            //a line yields value of io::Result<Option<String>>
            let line = line?;
            let event = serde_json::from_str(&line)?;
            Ok(event)
        })
    }
}
