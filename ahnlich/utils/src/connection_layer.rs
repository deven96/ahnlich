use std::sync::Arc;

use ahnlich_types::client::ConnectedClient;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tonic::Request;
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

pub struct RequestTracker<T> {
    inner: Request<T>,
    client_handler: Arc<ClientHandler>,
    connected_client: ConnectedClient,
}

impl<T> Drop for RequestTracker<T> {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}

#[derive(Clone)]
pub struct RequestTrackerLayer {
    client_handler: Arc<ClientHandler>,
}

impl RequestTrackerLayer {
    pub fn new(client_handler: Arc<ClientHandler>) -> Self {
        Self { client_handler }
    }
}

impl<S> tower::layer::Layer<S> for RequestTrackerLayer {
    type Service = RequestTrackerService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RequestTrackerService {
            inner,
            client_handler: Arc::clone(&self.client_handler),
        }
    }
}

pub struct RequestTrackerService<S> {
    inner: S,
    client_handler: Arc<ClientHandler>,
}

impl<S, ReqBody, ResBody> tower::Service<tonic::Request<ReqBody>> for RequestTrackerService<S>
where
    S: tower::Service<
            tonic::Request<RequestTracker<ReqBody>>,
            Response = tonic::Response<ResBody>,
            Error = tonic::Status,
        > + Clone
        + 'static, //ReqBody: fmt::Debug,
    ReqBody: 'static,
{
    type Response = S::Response;

    type Error = S::Error;
    //type Future = S::Future;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<ReqBody>) -> Self::Future {
        match req.remote_addr() {
            Some(addr) => {
                if let Some(connected_client) = self.client_handler.connect(addr) {
                    let tracked_request = RequestTracker {
                        inner: req,
                        client_handler: Arc::clone(&self.client_handler),
                        connected_client,
                    };
                    let mut inner = self.inner.clone();
                    Box::pin(async move { inner.call(Request::new(tracked_request)).await })
                //self.inner.call(Request::new(tracked_request))
                } else {
                    Box::pin(async move {
                        Err(tonic::Status::resource_exhausted(
                            "Max Connected Clients Reached",
                        ))
                    })
                }
            }
            None => {
                Box::pin(async move { Err(tonic::Status::aborted("Client Connection Aborted")) })
            }
        }
    }
}
