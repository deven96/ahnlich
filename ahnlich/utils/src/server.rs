use crate::allocator::GLOBAL_ALLOCATOR;
use crate::client::ClientHandler;
use crate::parallel;
use crate::persistence::AhnlichPersistenceUtils;
use crate::persistence::Persistence;
use async_trait::async_trait;
use futures::Stream;
use grpc_types::client::ConnectedClient;
use pin_project::{pin_project, pinned_drop};
use std::fmt::Debug;
use std::io::ErrorKind;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::atomic::AtomicBool;
use std::task::Context;
use std::task::Poll;
use std::{io::Result as IoResult, sync::Arc};
use task_manager::BlockingTask;
use task_manager::TaskManager;
use tokio::io::AsyncRead;
use tokio::io::AsyncWrite;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_util::sync::CancellationToken;
use tonic::transport::server::Connected;
use tonic::transport::server::TcpConnectInfo;

#[derive(Debug)]
pub struct ServerUtilsConfig<'a> {
    pub service_name: &'static str,
    // persistence stuff
    pub persistence_interval: u64,
    pub persist_location: &'a Option<std::path::PathBuf>,
    // global allocator
    pub allocator_size: usize,
    pub threadpool_size: usize,
}

#[async_trait]
pub trait AhnlichServerUtils: BlockingTask + Sized + Send + Sync + 'static + Debug {
    type PersistenceTask: AhnlichPersistenceUtils;

    fn config(&self) -> ServerUtilsConfig;

    fn store_handler(&self) -> &Arc<Self::PersistenceTask>;
    fn write_flag(&self) -> Arc<AtomicBool> {
        self.store_handler().write_flag()
    }

    fn cancellation_token(&self) -> CancellationToken;

    fn task_manager(&self) -> Arc<TaskManager>;

    /// Runs through several processes to start up the server
    /// - Sets global allocator cap
    /// - Spawns Persistence listeneer thread
    /// - Accepts incoming connections to the listener and processes streams
    /// - Listens for ctrl_c signal to trigger spawned tasks cancellation
    /// - Cancellation triggers clean up of loggers and tracers
    async fn start(self) -> IoResult<()> {
        let service_name = self.config().service_name;
        let global_allocator_cap = self.config().allocator_size;
        // WARNING: `set_limit` fails if the global allocator has already allocated memory beyond
        // the size being set, therefore might point to a need to bump up the default
        // `allocator_size`
        GLOBAL_ALLOCATOR
            .set_limit(global_allocator_cap)
            .unwrap_or_else(|_| panic!("Could not set up {service_name} with allocator_size"));
        log::debug!("Set max size for global allocator to: {global_allocator_cap}");
        parallel::init_threadpool(self.config().threadpool_size);
        let task_manager = self.task_manager();

        if let Some(persist_location) = self.config().persist_location {
            let persistence_task = Persistence::task(
                self.write_flag(),
                self.config().persistence_interval,
                persist_location,
                self.store_handler().get_snapshot(),
            );
            task_manager.spawn_task_loop(persistence_task).await;
        };
        task_manager.spawn_blocking(self).await;
        task_manager.wait().await;
        tracer::shutdown_tracing();
        log::info!("Shutdown complete");
        Ok(())
    }
}

#[derive(Debug)]
pub struct CustomTcpListenerStream {
    inner: TcpListener,
    client_handler: Arc<ClientHandler>,
}

impl CustomTcpListenerStream {
    pub fn new(listener: TcpListener, client_handler: Arc<ClientHandler>) -> Self {
        Self {
            inner: listener,
            client_handler,
        }
    }
}

// We need pin project to ensure that the inner TcpStream can be safely pinned
#[pin_project(PinnedDrop)]
pub struct CustomTcpStream {
    #[pin]
    inner: TcpStream,
    connected_client: Option<ConnectedClient>,
    client_handler: Arc<ClientHandler>,
}

impl AsyncRead for CustomTcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        self.project().inner.poll_read(cx, buf)
    }
}

impl AsyncWrite for CustomTcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, std::io::Error>> {
        self.project().inner.poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_flush(cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Result<(), std::io::Error>> {
        self.project().inner.poll_shutdown(cx)
    }
}

impl Connected for CustomTcpStream {
    type ConnectInfo = TcpConnectInfo;

    fn connect_info(&self) -> Self::ConnectInfo {
        self.inner.connect_info()
    }
}

#[pinned_drop]
impl PinnedDrop for CustomTcpStream {
    fn drop(mut self: Pin<&mut Self>) {
        if let Some(connected_client) = self.as_mut().project().connected_client.take() {
            self.project().client_handler.disconnect(&connected_client);
        }
    }
}

impl Stream for CustomTcpListenerStream {
    type Item = std::io::Result<CustomTcpStream>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<std::io::Result<CustomTcpStream>>> {
        match self.inner.poll_accept(cx) {
            Poll::Ready(Ok((stream, _))) => {
                let peer_addr = match stream.peer_addr() {
                    Ok(addr) => addr,
                    Err(e) => return Poll::Ready(Some(Err(e))),
                };
                if let Some(connected_client) = self.client_handler.connect(peer_addr) {
                    Poll::Ready(Some(Ok(CustomTcpStream {
                        inner: stream,
                        client_handler: self.client_handler.clone(),
                        connected_client: Some(connected_client),
                    })))
                } else {
                    Poll::Ready(Some(Err(std::io::Error::new(
                        ErrorKind::ConnectionAborted,
                        "Max Connected Clients Reached",
                    ))))
                }
            }
            Poll::Ready(Err(err)) => Poll::Ready(Some(Err(err))),
            Poll::Pending => Poll::Pending,
        }
    }
}

impl AsRef<TcpListener> for CustomTcpListenerStream {
    fn as_ref(&self) -> &TcpListener {
        &self.inner
    }
}

#[derive(Debug)]
pub enum ListenerStreamOrAddress {
    ListenerStream(CustomTcpListenerStream),
    Address(SocketAddr),
}

impl ListenerStreamOrAddress {
    // new always creates a TcpListenerStream variant to be taken
    pub async fn new(addr: String, client_handler: Arc<ClientHandler>) -> IoResult<Self> {
        Ok(ListenerStreamOrAddress::ListenerStream(
            CustomTcpListenerStream::new(
                tokio::net::TcpListener::bind(addr).await?,
                client_handler,
            ),
        ))
    }
    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        match &self {
            Self::ListenerStream(stream) => stream.as_ref().local_addr(),
            Self::Address(addr) => Ok(addr.clone()),
        }
    }
}
