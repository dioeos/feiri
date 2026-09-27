use std::{
    fs::{File, OpenOptions},
    io::ErrorKind,
    os::unix::fs::FileTypeExt,
    path::PathBuf,
    sync::Arc,
};

use feiri_ipc::Request;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tracing::{info, warn};

use crate::handlers::ActionHandler;

use super::error::Error;

pub struct IpcServer {
    listener: UnixListener,
    action_handler: ActionHandler,
    _lock: File,
}

impl IpcServer {
    pub async fn new(path: PathBuf, action_handler: ActionHandler) -> Result<Self, Error> {
        let mut lock_name = path.as_os_str().to_os_string();
        lock_name.push(".lock");

        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(PathBuf::from(lock_name))
            .map_err(Error::FailedToOpenLockFile)?;

        lock.try_lock().map_err(Error::FailedToAcquireLock)?;

        let listener = match UnixListener::bind(&path) {
            Ok(listener) => listener,

            Err(err) if err.kind() == ErrorKind::AddrInUse => {
                match UnixStream::connect(&path).await {
                    Ok(_) => {
                        return Err(Error::ServerAddrInUse { addr: path });
                    }

                    Err(err) if err.kind() == ErrorKind::ConnectionRefused => {
                        let metadata = std::fs::symlink_metadata(&path)
                            .map_err(Error::FailedToReadSocketMetadata)?;

                        if !metadata.file_type().is_socket() {
                            return Err(Error::UnexpectedSocketFileConflict {
                                file_path: path,
                                reason: "socket path is not a socket",
                            });
                        }

                        std::fs::remove_file(&path).map_err(Error::FailedToRemoveStaleSocket)?;

                        UnixListener::bind(&path).map_err(Error::FailedToBindSocket)?
                    }

                    Err(err) => {
                        return Err(Error::UnexpectedIpcServerFailure(err.to_string()));
                    }
                }
            }

            Err(err) => {
                return Err(Error::UnexpectedIpcServerFailure(err.to_string()));
            }
        };

        Ok(Self {
            listener,
            action_handler,
            _lock: lock,
        })
    }

    pub async fn run(self: Arc<Self>) -> Result<(), Error> {
        info!("running ipc server");

        loop {
            let (stream, _) = self
                .listener
                .accept()
                .await
                .map_err(Error::FailedToAcceptIncomingConnection)?;

            let server = Arc::clone(&self);

            tokio::spawn(async move {
                if let Err(err) = server.handle_client(stream).await {
                    warn!(?err, "IPC client failed");
                }
            });
        }
    }

    async fn handle_client(&self, stream: UnixStream) -> Result<(), Error> {
        let mut reader = BufReader::new(stream);
        let mut buf = String::new();

        let Some(request) = Self::read_stream(&mut reader, &mut buf).await? else {
            return Ok(());
        };

        match request {
            Request::Action(action) => {
                self.action_handler.handle_action_request(action).await?;
            }
        }
        Ok(())
    }

    async fn read_stream(
        reader: &mut BufReader<UnixStream>,
        buf: &mut String,
    ) -> Result<Option<feiri_ipc::Request>, Error> {
        buf.clear();

        let bytes_read = reader
            .read_line(buf)
            .await
            .map_err(Error::FailedToReadStreamRequest)?;

        if bytes_read == 0 {
            return Ok(None);
        }

        let request: feiri_ipc::Request =
            serde_json::from_str(buf).map_err(Error::FailedToSerializeRequest)?;

        Ok(Some(request))
    }
}
