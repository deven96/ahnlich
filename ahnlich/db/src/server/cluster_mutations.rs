use crate::server::cluster::ClusterRuntime;
use crate::server::cluster_forwarding::forward_to_leader;
use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::types::{DbCommand, DbResponse};
use openraft::error::RaftError;
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};

macro_rules! command_name {
    ($ty:ty) => {{
        let full = stringify!($ty);
        full.rsplit("::").next().unwrap_or(full)
    }};
}

macro_rules! submit_db_command {
    ($cluster:expr, $metadata:expr, $query_ty:ty, $params:expr, $builder:path, $forward:expr) => {{
        crate::server::cluster_mutations::submit_db_command_inner(
            $cluster, $metadata, $params, $builder, $forward,
        )
    }};
}
pub(crate) use submit_db_command;

enum ClientWriteFailure {
    ForwardToLeader(String),
    Status(tonic::Status),
}

fn map_client_write_error(
    command_name: &str,
    err: RaftError<u64, openraft::error::ClientWriteError<u64, ReplicationNode>>,
) -> ClientWriteFailure {
    if let Some(forward) = err.forward_to_leader() {
        if let Some(leader) = forward.leader_node.as_ref() {
            ClientWriteFailure::ForwardToLeader(leader.service_addr.clone())
        } else {
            ClientWriteFailure::Status(tonic::Status::failed_precondition(format!(
                "{command_name} requires leader routing, but no leader is known",
            )))
        }
    } else {
        ClientWriteFailure::Status(tonic::Status::internal(format!(
            "{command_name} raft write failed: {err}"
        )))
    }
}

async fn submit_raw_db_command(
    cluster: Option<&ClusterRuntime>,
    command_name: &str,
    command: DbCommand,
) -> Result<DbResponse, ClientWriteFailure> {
    let cluster = cluster.ok_or_else(|| {
        ClientWriteFailure::Status(tonic::Status::failed_precondition(
            "DB server is not running in cluster mode",
        ))
    })?;

    cluster
        .raft
        .client_write(command)
        .await
        .map(|response| response.data)
        .map_err(|err| map_client_write_error(command_name, err))
}

pub(crate) async fn submit_db_command_inner<Q, T, F, Fut>(
    cluster: Option<&ClusterRuntime>,
    metadata: tonic::metadata::MetadataMap,
    params: Q,
    command_builder: impl FnOnce(Vec<u8>) -> DbCommand,
    forward: F,
) -> Result<T, tonic::Status>
where
    Q: prost::Message,
    T: DeserializeOwned + 'static,
    F: FnOnce(
        ahnlich_types::services::db_service::db_service_client::DbServiceClient<
            tonic::transport::Channel,
        >,
        tonic::Request<Q>,
    ) -> Fut,
    Fut: std::future::Future<Output = Result<tonic::Response<T>, tonic::Status>>,
{
    let command_name = command_name!(Q);
    let command = command_builder(params.encode_to_vec());

    match submit_raw_db_command(cluster, command_name, command).await {
        Err(ClientWriteFailure::ForwardToLeader(leader_addr)) => {
            let cluster = cluster.expect("cluster was required before forwarding");
            forward_to_leader(cluster, &leader_addr, metadata, params, forward).await
        }
        Err(ClientWriteFailure::Status(status)) => Err(status),
        Ok(DbResponse::Bytes(bytes)) => bitcode::deserialize(bytes.as_slice()).map_err(|err| {
            tonic::Status::internal(format!("failed to decode {command_name} raw result: {err}",))
        }),
        Ok(DbResponse::Unit) if TypeId::of::<T>() == TypeId::of::<()>() => {
            let unit: Box<dyn Any> = Box::new(());
            Ok(*unit
                .downcast::<T>()
                .expect("unit type checked before downcast"))
        }
        Ok(DbResponse::Unit) => Err(tonic::Status::internal(format!(
            "{command_name} unexpectedly returned unit response",
        ))),
    }
}
