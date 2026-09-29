use std::{env::var_os, io, path::PathBuf};

use tokio::{io::AsyncWriteExt, net::UnixStream};

use crate::Request;

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
    stream: UnixStream,
}

impl Socket {
    pub async fn connect() -> Result<Self, Error> {
        let xdg_os_string = var_os("XDG_RUNTIME_DIR").ok_or(Error::MissingRuntimeDir)?;

        Ok(Self::connect_to(PathBuf::from(xdg_os_string).join(FEIRI_IPC_SOCK)).await?)
    }

    pub async fn connect_to(path: PathBuf) -> io::Result<Self> {
        // let stream = UnixStream::connect(path).await?;
        // let stream = BufReader::new(stream);
        // Ok(Self { stream })
        Ok(Self {
            stream: UnixStream::connect(path).await?,
        })
    }

    pub async fn send(&mut self, request: Request) -> Result<(), Error> {
        let mut bytes = serde_json::to_vec(&request)?;
        bytes.push(b'\n');
        self.stream.write_all(&bytes).await?;
        self.stream.shutdown().await?;
        Ok(())
    }
}
