use clap::Parser;
use grpc_types::services::db_service::db_service_server::DbServiceServer;

use std::{error::Error, sync::Arc};
use utils::{
    cli::validate_persistence, connection_layer::RequestTrackerLayer, server::AhnlichServerUtils,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = ahnlich_db::cli::Cli::parse();
    match &cli.command {
        ahnlich_db::cli::Commands::Run(config) => {
            if config.common.enable_persistence {
                validate_persistence(
                    config.common.allocator_size,
                    config.common.persist_location.as_ref(),
                )?;
            }
            let server = ahnlich_db::server::handler::Server::new(config).await?;
            let conn_tracer = RequestTrackerLayer::new(Arc::clone(&server.client_handler()));
            let db_service = DbServiceServer::new(server);

            //server.start().await?;

            tonic::transport::Server::builder()
                .layer(conn_tracer)
                .add_service(db_service)
                .serve("localhost:8000".parse().expect("Failed to parse"))
                .await;
        }
    }
    Ok(())
}
