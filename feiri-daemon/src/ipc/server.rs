use std::{
    fs::{File, OpenOptions},
    io::{self, ErrorKind},
    os::unix::fs::FileTypeExt,
    path::PathBuf,
    sync::Arc,
};

use feiri_ipc::{Command, Reply, Response};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
};
use tracing::{debug, info, warn};

use crate::handlers::{ActionHandler, QueryHandler};

use super::error::Error;

pub struct IpcServer {
    listener: UnixListener,
    action_handler: ActionHandler,
    query_handler: QueryHandler,
    event_sender: tokio::sync::broadcast::Sender<feiri_ipc::Event>,
    _lock: File,
}

impl IpcServer {
    pub async fn new(
        path: PathBuf,
        action_handler: ActionHandler,
        query_handler: QueryHandler,
        event_sender: tokio::sync::broadcast::Sender<feiri_ipc::Event>,
    ) -> Result<Self, Error> {
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
            query_handler,
            event_sender,
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

            let mut buf = Vec::new();
            let mut reader = BufReader::new(stream);

            let res = reader.read_until(b'\n', &mut buf).await;
            //continue to keep server alive when client disconnects / no data to read
            match res {
                //client closed connection
                Ok(0) => continue,
                Ok(_) => (),
                //normal client disconnect
                Err(err) if err.kind() == io::ErrorKind::BrokenPipe => continue,
                Err(err) => {
                    return Err(Error::FailedToReadClientRequest(err));
                }
            }

            let request: feiri_ipc::Request =
                serde_json::from_slice(&buf).map_err(Error::FailedToSerializeRequest)?;

            let stream = reader.into_inner();

            let server = Arc::clone(&self);

            match request {
                feiri_ipc::Request::EventStream => {
                    tokio::spawn(async move {
                        if let Err(err) = server.handle_event_stream(stream).await {
                            warn!(?err, "IPC event stream failed");
                        }
                    });
                }
                feiri_ipc::Request::Operation(operation) => {
                    tokio::spawn(async move {
                        if let Err(err) = server.handle_command(stream, operation).await {
                            warn!(?err, "IPC operation failed");
                        }
                    });
                }
            }
        }
    }

    async fn handle_event_stream(&self, mut stream: UnixStream) -> Result<(), Error> {
        let mut daemon_evt_rx = self.event_sender.subscribe();

        let ack = Response::Handled;
        let mut bytes =
            serde_json::to_vec(&Ok::<_, String>(ack)).map_err(Error::FailedToConvertMsgToBytes)?;

        bytes.push(b'\n');
        stream
            .write_all(&bytes)
            .await
            .map_err(Error::FailedToAckSuccessfulEventStream)?;

        loop {
            let recv_res = daemon_evt_rx.recv().await;
            let event: feiri_ipc::Event = match recv_res {
                Ok(event) => event,
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    warn!("receiver lagged");
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    break;
                }
            };
            let mut msg = serde_json::to_vec(&event).map_err(Error::FailedToConvertMsgToBytes)?;

            msg.push(b'\n');
            stream
                .write_all(&msg)
                .await
                .map_err(Error::FailedToWriteEvent)?;
        }

        todo!()
    }

    //Returns an `Err(String)`, which contains a `Service` error
    async fn handle_command(
        &self,
        mut stream: UnixStream,
        operation: feiri_ipc::Command,
    ) -> Result<(), Error> {
        let reply: Reply = match operation {
            Command::Action(action) => {
                match self.action_handler.handle_action_request(action).await {
                    Ok(()) => Ok(Response::Handled),
                    Err(err) => Err(err.to_string()),
                }
            }
            Command::Query(query) => match self.query_handler.handle_query_request(query).await {
                Ok(result) => Ok(Response::Query(result)),
                Err(err) => Err(err.to_string()),
            },
        };

        let mut bytes = serde_json::to_vec(&reply).map_err(Error::FailedToConvertMsgToBytes)?;

        bytes.push(b'\n');
        stream
            .write_all(&bytes)
            .await
            .map_err(Error::FailedToWriteResponse)?;

        Ok(())
    }
}
