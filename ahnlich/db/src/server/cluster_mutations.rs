use crate::server::cluster::ClusterRuntime;
use ahnlich_replication::node::ReplicationNode;
use ahnlich_replication::types::{DbCommand, DbResponse};
use openraft::error::RaftError;
use serde::de::DeserializeOwned;
use std::any::{Any, TypeId};

pub(crate) fn map_client_write_error(
    command_name: &str,
    err: RaftError<u64, openraft::error::ClientWriteError<u64, ReplicationNode>>,
) -> tonic::Status {
    if let Some(forward) = err.forward_to_leader() {
        let leader = forward
            .leader_node
            .as_ref()
            .map(|node| node.service_addr.as_str())
            .unwrap_or("unknown leader");
        tonic::Status::failed_precondition(format!(
            "{command_name} requires leader routing; retry against {leader}",
        ))
    } else {
        tonic::Status::internal(format!("{command_name} raft write failed: {err}"))
    }
}

async fn submit_raw_db_command(
    cluster: Option<&ClusterRuntime>,
    command_name: &str,
    command: DbCommand,
) -> Result<DbResponse, tonic::Status> {
    let cluster = cluster.ok_or_else(|| {
        tonic::Status::failed_precondition("DB server is not running in cluster mode")
    })?;

    cluster
        .raft
        .client_write(command)
        .await
        .map(|response| response.data)
        .map_err(|err| map_client_write_error(command_name, err))
}

pub(crate) async fn submit_db_command<T>(
    cluster: Option<&ClusterRuntime>,
    command_name: &str,
    command: DbCommand,
) -> Result<T, tonic::Status>
where
    T: DeserializeOwned + 'static,
{
    match submit_raw_db_command(cluster, command_name, command).await? {
        DbResponse::Bytes(bytes) => bitcode::deserialize(bytes.as_slice()).map_err(|err| {
            tonic::Status::internal(format!("failed to decode {command_name} raw result: {err}",))
        }),
        DbResponse::Unit if TypeId::of::<T>() == TypeId::of::<()>() => {
            let unit: Box<dyn Any> = Box::new(());
            Ok(*unit
                .downcast::<T>()
                .expect("unit type checked before downcast"))
        }
        DbResponse::Unit => Err(tonic::Status::internal(format!(
            "{command_name} unexpectedly returned unit response",
        ))),
    }
}
