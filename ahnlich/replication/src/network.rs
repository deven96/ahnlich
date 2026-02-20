use std::error::Error;
use std::sync::Arc;

use bincode::{deserialize, serialize};
use openraft::error::{InstallSnapshotError, NetworkError, RPCError, RaftError};
use openraft::network::{RPCOption, RaftNetwork, RaftNetworkFactory};
use openraft::raft::{
    AppendEntriesRequest, AppendEntriesResponse, InstallSnapshotRequest, InstallSnapshotResponse,
    VoteRequest, VoteResponse,
};
use openraft::{Raft, RaftTypeConfig};
use tonic::{Request, Response, Status};

use crate::proto::raft_internal::raft_internal_service_client::RaftInternalServiceClient;
use crate::proto::raft_internal::raft_internal_service_server::RaftInternalService;
use crate::proto::raft_internal::{
    AppendEntriesRequest as PbAppendEntriesRequest,
    AppendEntriesResponse as PbAppendEntriesResponse,
    InstallSnapshotRequest as PbInstallSnapshotRequest,
    InstallSnapshotResponse as PbInstallSnapshotResponse, VoteRequest as PbVoteRequest,
    VoteResponse as PbVoteResponse,
};

#[derive(Debug, Clone)]
pub struct GrpcRaftNetwork<C: RaftTypeConfig> {
    target: String,
    _p: std::marker::PhantomData<C>,
}

impl<C: RaftTypeConfig> GrpcRaftNetwork<C> {
    pub fn new(target: String) -> Self {
        let target = if target.starts_with("http://") || target.starts_with("https://") {
            target
        } else {
            format!("http://{target}")
        };

        Self {
            target,
            _p: std::marker::PhantomData,
        }
    }

    async fn client(
        &self,
    ) -> Result<RaftInternalServiceClient<tonic::transport::Channel>, tonic::transport::Error> {
        RaftInternalServiceClient::connect(self.target.clone()).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct GrpcRaftNetworkFactory<C: RaftTypeConfig> {
    _p: std::marker::PhantomData<C>,
}

impl<C: RaftTypeConfig> RaftNetworkFactory<C> for GrpcRaftNetworkFactory<C>
where
    C::Node: Into<openraft::BasicNode> + Clone,
{
    type Network = GrpcRaftNetwork<C>;

    async fn new_client(&mut self, _target: C::NodeId, node: &C::Node) -> Self::Network {
        let basic: openraft::BasicNode = node.clone().into();
        GrpcRaftNetwork::new(basic.addr)
    }
}

fn rpc_err<C: RaftTypeConfig, E: Error + 'static>(
    err: E,
) -> RPCError<C::NodeId, C::Node, RaftError<C::NodeId>> {
    RPCError::Network(NetworkError::new(&err))
}

fn rpc_err_install<C: RaftTypeConfig, E: Error + 'static>(
    err: E,
) -> RPCError<C::NodeId, C::Node, RaftError<C::NodeId, InstallSnapshotError>> {
    RPCError::Network(NetworkError::new(&err))
}

impl<C: RaftTypeConfig> RaftNetwork<C> for GrpcRaftNetwork<C> {
    async fn append_entries(
        &mut self,
        rpc: AppendEntriesRequest<C>,
        _option: RPCOption,
    ) -> Result<AppendEntriesResponse<C::NodeId>, RPCError<C::NodeId, C::Node, RaftError<C::NodeId>>>
    {
        let mut client = self.client().await.map_err(rpc_err::<C, _>)?;
        let payload = PbAppendEntriesRequest {
            payload: serialize(&rpc).map_err(rpc_err::<C, _>)?,
        };
        let resp = client
            .append_entries(Request::new(payload))
            .await
            .map_err(rpc_err::<C, _>)?;
        let decoded: AppendEntriesResponse<C::NodeId> =
            deserialize(&resp.into_inner().payload).map_err(rpc_err::<C, _>)?;
        Ok(decoded)
    }

    async fn install_snapshot(
        &mut self,
        rpc: InstallSnapshotRequest<C>,
        _option: RPCOption,
    ) -> Result<
        InstallSnapshotResponse<C::NodeId>,
        RPCError<C::NodeId, C::Node, RaftError<C::NodeId, InstallSnapshotError>>,
    > {
        let mut client = self.client().await.map_err(rpc_err_install::<C, _>)?;
        let payload = PbInstallSnapshotRequest {
            payload: serialize(&rpc).map_err(rpc_err_install::<C, _>)?,
        };
        let resp = client
            .install_snapshot(Request::new(payload))
            .await
            .map_err(rpc_err_install::<C, _>)?;
        let decoded: InstallSnapshotResponse<C::NodeId> =
            deserialize(&resp.into_inner().payload).map_err(rpc_err_install::<C, _>)?;
        Ok(decoded)
    }

    async fn vote(
        &mut self,
        rpc: VoteRequest<C::NodeId>,
        _option: RPCOption,
    ) -> Result<VoteResponse<C::NodeId>, RPCError<C::NodeId, C::Node, RaftError<C::NodeId>>> {
        let mut client = self.client().await.map_err(rpc_err::<C, _>)?;
        let payload = PbVoteRequest {
            payload: serialize(&rpc).map_err(rpc_err::<C, _>)?,
        };
        let resp = client
            .vote(Request::new(payload))
            .await
            .map_err(rpc_err::<C, _>)?;
        let decoded: VoteResponse<C::NodeId> =
            deserialize(&resp.into_inner().payload).map_err(rpc_err::<C, _>)?;
        Ok(decoded)
    }
}

pub struct GrpcRaftService<C: RaftTypeConfig> {
    raft: Arc<Raft<C>>,
}

impl<C: RaftTypeConfig> GrpcRaftService<C> {
    pub fn new(raft: Arc<Raft<C>>) -> Self {
        Self { raft }
    }
}

#[tonic::async_trait]
impl<C: RaftTypeConfig> RaftInternalService for GrpcRaftService<C> {
    async fn append_entries(
        &self,
        request: Request<PbAppendEntriesRequest>,
    ) -> Result<Response<PbAppendEntriesResponse>, Status> {
        let rpc: AppendEntriesRequest<C> = deserialize(&request.into_inner().payload)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let resp = self
            .raft
            .append_entries(rpc)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PbAppendEntriesResponse {
            payload: serialize(&resp).map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn install_snapshot(
        &self,
        request: Request<PbInstallSnapshotRequest>,
    ) -> Result<Response<PbInstallSnapshotResponse>, Status> {
        let rpc: InstallSnapshotRequest<C> = deserialize(&request.into_inner().payload)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let resp = self
            .raft
            .install_snapshot(rpc)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PbInstallSnapshotResponse {
            payload: serialize(&resp).map_err(|e| Status::internal(e.to_string()))?,
        }))
    }

    async fn vote(
        &self,
        request: Request<PbVoteRequest>,
    ) -> Result<Response<PbVoteResponse>, Status> {
        let rpc: VoteRequest<C::NodeId> = deserialize(&request.into_inner().payload)
            .map_err(|e| Status::invalid_argument(e.to_string()))?;
        let resp = self
            .raft
            .vote(rpc)
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        Ok(Response::new(PbVoteResponse {
            payload: serialize(&resp).map_err(|e| Status::internal(e.to_string()))?,
        }))
    }
}

pub fn map_raft_error<E: ToString>(err: E) -> tonic::Status {
    tonic::Status::unavailable(err.to_string())
}
