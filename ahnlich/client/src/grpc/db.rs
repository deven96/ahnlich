use grpc_types::services::db_service::{
    db_service_client::DbServiceClient, db_service_server::DbServiceServer,
};
use tonic::transport::Channel;
// GRPC Client for Ahnlich DB
#[derive(Debug, Clone)]
pub struct DbClient {
    db_service_client: DbServiceClient<Channel>,
}
