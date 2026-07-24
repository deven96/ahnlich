use crate::server::cluster::ClusterRuntime;
use ahnlich_types::services::db_service::db_service_client::DbServiceClient;
use std::future::Future;
use tonic::metadata::MetadataMap;
use tonic::transport::Channel;

pub(crate) fn forwarded_request<T>(metadata: MetadataMap, params: T) -> tonic::Request<T> {
    let mut request = tonic::Request::new(params);
    *request.metadata_mut() = metadata;
    request
}

pub(crate) async fn forward_to_leader<Q, R, F, Fut>(
    cluster: &ClusterRuntime,
    leader_addr: &str,
    metadata: MetadataMap,
    params: Q,
    forward: F,
) -> Result<R, tonic::Status>
where
    F: FnOnce(DbServiceClient<Channel>, tonic::Request<Q>) -> Fut,
    Fut: Future<Output = Result<tonic::Response<R>, tonic::Status>>,
{
    let client = cluster
        .leader_client(leader_addr)
        .map_err(tonic::Status::internal)?;
    forward(client, forwarded_request(metadata, params))
        .await
        .map(tonic::Response::into_inner)
}
