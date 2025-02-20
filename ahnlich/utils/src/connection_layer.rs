use std::sync::Arc;

use ahnlich_types::client::ConnectedClient;
use hyper::Request;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tower_layer::Layer;

use crate::client::ClientHandler;

use std::{
    collections::HashSet,
    error::Error,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tower::Service;

#[derive(Clone)]
pub struct ConnectionTracker {
    active_connections: Arc<ClientHandler>,
}

impl ConnectionTracker {
    pub fn new(active_connections: Arc<ClientHandler>) -> Self {
        Self { active_connections }
    }
}

impl<S> Layer<S> for ConnectionTracker {
    type Service = ConnectionTrackingService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ConnectionTrackingService {
            inner,
            active_connections: self.active_connections.clone(),
        }
    }
}

pub struct ConnectionTrackingService<S> {
    inner: S,
    active_connections: Arc<ClientHandler>,
}

impl<S> Service<TcpStream> for ConnectionTrackingService<S>
where
    S: Service<TcpStream> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = ConnectionGuard<S::Response>;
    type Error = ConnectionTrackingServiceError;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner
            .poll_ready(cx)
            .map_err(|_| ConnectionTrackingServiceError::ConnectionLimitExceeded)
    }

    //Request<http_body_util::combinators::bo
    //x_body::UnsyncBoxBody<tonic::codegen::Bytes, Status>>>` is not implemented for `ConnectionTrackingService<Routes>`

    fn call(&mut self, req: Request) -> Self::Future {
        let remote_addr = req.peer_addr();

        match remote_addr {
            Ok(addr) => {
                if let Some(connected_client) = self.active_connections.connect(addr) {
                    let future = self.inner.call(req);
                    let active_connections_clone = Arc::clone(&self.active_connections);

                    Box::pin(async move {
                        let response = future
                            .await
                            .map_err(|_| ConnectionTrackingServiceError::ConnectionLimitExceeded)?;
                        Ok(ConnectionGuard {
                            response,
                            connected_client,
                            client_handler: active_connections_clone,
                        })
                    })
                } else {
                    return Box::pin(async {
                        Err(ConnectionTrackingServiceError::ConnectionLimitExceeded)
                    });
                }
            }
            Err(_err) => Box::pin(async { Err(ConnectionTrackingServiceError::PeerAddressError) }),
        }
    }
}

/// Custom error type for exceeding max connections
#[derive(Debug)]
pub enum ConnectionTrackingServiceError {
    ConnectionLimitExceeded,
    PeerAddressError,
}

impl fmt::Display for ConnectionTrackingServiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output_string = match self {
            ConnectionTrackingServiceError::ConnectionLimitExceeded => "Connection Limit Exceeded",
            ConnectionTrackingServiceError::PeerAddressError => "Peer Address Error",
        };
        write!(f, "{output_string}")
    }
}

impl Error for ConnectionTrackingServiceError {}

pub struct ConnectionGuard<T> {
    client_handler: Arc<ClientHandler>,
    connected_client: ConnectedClient,
    response: T,
}

impl<T> Drop for ConnectionGuard<T> {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}

impl<T> Service<hyper::Request<hyper::body::Incoming>> for ConnectionGuard<T>
where
    T: Service<hyper::Request<hyper::body::Incoming>>,
{
    type Response = T::Response;
    type Error = T::Error;
    type Future = T::Future;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.response.poll_ready(cx)
    }

    fn call(&mut self, req: hyper::Request<hyper::body::Incoming>) -> Self::Future {
        self.response.call(req)
    }
}
