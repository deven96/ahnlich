use crate::engine::store::StoreHandler;
use crate::errors::ServerError;
use crate::server::cluster::ClusterRuntime;
use std::net::SocketAddr;
use std::sync::Arc;

#[derive(Debug)]
pub(crate) enum StoreRuntime {
    Standalone(Arc<StoreHandler>),
    Cluster(ClusterRuntime),
}

impl StoreRuntime {
    pub(crate) fn cluster(&self) -> Option<&ClusterRuntime> {
        match self {
            Self::Standalone(_) => None,
            Self::Cluster(cluster) => Some(cluster),
        }
    }

    pub(crate) fn standalone_store_handler(&self) -> Option<&Arc<StoreHandler>> {
        match self {
            Self::Standalone(store_handler) => Some(store_handler),
            Self::Cluster(_) => None,
        }
    }

    #[allow(clippy::result_large_err)]
    pub(crate) fn with_store_handler<R>(
        &self,
        f: impl FnOnce(&StoreHandler) -> Result<R, ServerError>,
    ) -> Result<R, tonic::Status> {
        match self {
            Self::Standalone(store_handler) => f(store_handler).map_err(Into::into),
            Self::Cluster(cluster) => cluster
                .state_machine
                .with_handler(|handler| f(handler.store_handler()))
                .map_err(|err| {
                    tonic::Status::internal(format!(
                        "failed to access clustered state machine: {err}"
                    ))
                })?
                .map_err(Into::into),
        }
    }

    pub(crate) fn cluster_local_addr(&self) -> Option<SocketAddr> {
        self.cluster().map(|cluster| cluster.raft_addr)
    }
}
