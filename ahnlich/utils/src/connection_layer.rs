use std::sync::Arc;

use grpc_types::utils::TRACE_HEADER;
use tonic::{body::BoxBody, transport::server::TcpConnectInfo};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::client::ClientHandler;

use std::{
    error::Error,
    fmt,
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Clone)]
pub struct RequestTrackerLayer {
    client_handler: Arc<ClientHandler>,
}

impl RequestTrackerLayer {
    pub fn new(client_handler: Arc<ClientHandler>) -> Self {
        Self { client_handler }
    }
}

impl<S: Clone> tower::layer::Layer<S> for RequestTrackerLayer {
    type Service = RequestTrackerService<S>;
    fn layer(&self, inner: S) -> Self::Service {
        RequestTrackerService {
            inner,
            client_handler: Arc::clone(&self.client_handler),
        }
    }
}

#[derive(Clone)]
pub struct RequestTrackerService<S: Clone> {
    inner: S,
    client_handler: Arc<ClientHandler>,
}

impl<S, ResBody> tower::Service<http::Request<BoxBody>> for RequestTrackerService<S>
where
    S: tower::Service<
            http::Request<BoxBody>,
            Response = http::Response<ResBody>,
            Error = Box<dyn std::error::Error + Send + Sync>,
        > + Clone
        + Send
        + 'static,
    S::Future: Send,
{
    type Response = S::Response;

    type Error = Box<dyn std::error::Error + Send + Sync>;
    //type Future = S::Future;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: http::Request<BoxBody>) -> Self::Future {
        // TODO: Check if extensions actually works this way and if we get the tcpstream back
        match req
            .extensions()
            .get::<TcpConnectInfo>()
            .and_then(|a| a.remote_addr())
        {
            Some(addr) => {
                if let Some(connected_client) = self.client_handler.connect(addr) {
                    let client_handler = self.client_handler.clone();
                    let mut inner = self.inner.clone();
                    let fut = inner.call(req);
                    Box::pin(async move {
                        let res = fut.await;
                        client_handler.disconnect(&connected_client);
                        res
                    })
                //self.inner.call(Request::new(tracked_request))
                } else {
                    Box::pin(async move {
                        Err(status_to_error(tonic::Status::resource_exhausted(
                            "Max Connected Clients Reached",
                        )))
                    })
                }
            }
            None => Box::pin(async move {
                Err(status_to_error(tonic::Status::aborted(
                    "Client Connection Aborted",
                )))
            }),
        }
    }
}

struct TonicError(tonic::Status);

impl fmt::Display for TonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.0.code(), self.0.message())
    }
}

impl fmt::Debug for TonicError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "TonicError {{ code: {:?}, message: {} }}",
            self.0.code(),
            self.0.message()
        )
    }
}

impl Error for TonicError {}

impl From<tonic::Status> for TonicError {
    fn from(status: tonic::Status) -> Self {
        TonicError(status)
    }
}

fn status_to_error(status: tonic::Status) -> Box<dyn Error + Send + Sync> {
    Box::new(TonicError::from(status))
}

pub fn trace_with_parent(req: &http::Request<()>) -> tracing::Span {
    let span = tracing::info_span!("query-processor");
    if let Some(trace_parent) = req
        .headers()
        .get(TRACE_HEADER)
        .and_then(|val| val.to_str().ok())
    {
        if let Ok(parent_context) = tracer::trace_parent_to_span(trace_parent) {
            span.set_parent(parent_context);
        };
    }
    span
}
