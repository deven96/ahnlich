use ahnlich_types::{
    metadata::MetadataValue,
    query_builders::db::{self as db_params},
};
use grpc_types::{
    algorithm::{algorithms::Algorithm, nonlinear::NonLinearAlgorithm},
    db::{
        pipeline::db_query::Query,
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, DelKey, DelPred,
            DropNonLinearAlgorithmIndex, DropPredIndex, DropStore, GetKey, GetPred, GetSimN,
            InfoServer, ListClients, ListStores, Ping, Set, StoreEntry,
        },
        server::{
            ClientList, CreateIndex, Del, Get, GetSimN as GetSimNResult, Pong, Set as SetResult,
            StoreList, Unit,
        },
    },
    keyval::StoreKey,
    metadata::{MetadataType, MetadataValue as GrpcMetadataValue},
    services::db_service::db_service_client::DbServiceClient,
    shared::info::ServerInfo,
    utils::{add_trace_parent, from_internal_create_store, to_grpc_predicate_condition},
};
use tonic::transport::Channel;

use crate::error::AhnlichError;

#[derive(Debug, Clone)]
pub struct DbPipeline {
    queries: Vec<Query>,
    client: DbServiceClient<Channel>,
}
// TODO: implement DbPipeline
impl DbPipeline {}
// GRPC Client for Ahnlich DB
//
// client needs &mut as it can only send one request in flight, hence is not thread safe to use,
// however `Channel` makes use of `tower_buffer::Buffer` underneath and hence DBClient is cheap
// to clone and is encouraged for use across multiple threads
// https://docs.rs/tonic/latest/tonic/transport/struct.Channel.html#multiplexing-requests
#[derive(Debug, Clone)]
pub struct DbClient {
    client: DbServiceClient<Channel>,
}

impl DbClient {
    pub async fn new(addr: String) -> Result<Self, AhnlichError> {
        let addr = if !(addr.starts_with("https://") || addr.starts_with("http://")) {
            format!("http://{addr}")
        } else {
            addr
        };
        let channel = Channel::from_shared(addr)?;
        let client = DbServiceClient::connect(channel).await?;
        Ok(Self { client })
    }

    pub async fn create_store(
        &mut self,
        params: db_params::CreateStoreParams,
    ) -> Result<Unit, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(from_internal_create_store(params));
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.create_store(req).await?.into_inner())
    }

    pub async fn create_pred_index(
        &mut self,
        params: db_params::CreatePredIndexParams,
    ) -> Result<CreateIndex, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(CreatePredIndex {
            store: params.store.0,
            predicates: params
                .predicates
                .into_iter()
                .map(|a| a.to_string())
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.create_pred_index(req).await?.into_inner())
    }

    pub async fn create_non_linear_algorithm_index(
        &mut self,
        params: db_params::CreateNonLinearAlgorithmIndexParams,
    ) -> Result<CreateIndex, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(CreateNonLinearAlgorithmIndex {
            store: params.store.0,
            non_linear_indices: params
                .non_linear_indices
                .into_iter()
                .filter_map(|a| NonLinearAlgorithm::from(a).try_into().ok())
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .create_non_linear_algorithm_index(req)
            .await?
            .into_inner())
    }

    pub async fn get_key(&mut self, params: db_params::GetKeyParams) -> Result<Get, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(GetKey {
            store: params.store.0,
            keys: params
                .keys
                .into_iter()
                .map(|key| StoreKey { key: key.0 })
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.get_key(req).await?.into_inner())
    }

    pub async fn get_pred(
        &mut self,
        params: db_params::GetPredParams,
    ) -> Result<Get, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(GetPred {
            store: params.store.0,
            condition: to_grpc_predicate_condition(params.condition).map(|a| *a),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.get_pred(req).await?.into_inner())
    }

    pub async fn get_sim_n(
        &mut self,
        params: db_params::GetSimNParams,
    ) -> Result<GetSimNResult, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(GetSimN {
            store: params.store.0,
            search_input: Some(StoreKey {
                key: params.search_input.0,
            }),
            closest_n: params.closest_n.get() as u64,
            algorithm: Algorithm::from(params.algorithm)
                .try_into()
                .expect("Should convert algorithm"),
            condition: params
                .condition
                .map(|cond| to_grpc_predicate_condition(cond).map(|a| *a))
                .flatten(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.get_sim_n(req).await?.into_inner())
    }

    pub async fn set(&mut self, params: db_params::SetParams) -> Result<SetResult, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(Set {
            store: params.store.0,
            inputs: params
                .inputs
                .into_iter()
                .map(|(k, v)| StoreEntry {
                    key: Some(StoreKey { key: k.0 }),
                    value: v
                        .into_iter()
                        .map(|(k, v)| {
                            (
                                k.to_string(),
                                match v {
                                    MetadataValue::RawString(text) => GrpcMetadataValue {
                                        r#type: MetadataType::RawString.into(),
                                        value: Some(
                                            grpc_types::metadata::metadata_value::Value::RawString(
                                                text,
                                            ),
                                        ),
                                    },
                                    MetadataValue::Image(bin) => GrpcMetadataValue {
                                        r#type: MetadataType::Image.into(),
                                        value: Some(
                                            grpc_types::metadata::metadata_value::Value::Image(bin),
                                        ),
                                    },
                                },
                            )
                        })
                        .collect(),
                })
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.set(req).await?.into_inner())
    }

    pub async fn drop_pred_index(
        &mut self,
        params: db_params::DropPredIndexParams,
    ) -> Result<Del, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(DropPredIndex {
            store: params.store.0,
            predicates: params
                .predicates
                .into_iter()
                .map(|a| a.to_string())
                .collect(),
            error_if_not_exists: params.error_if_not_exists,
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.drop_pred_index(req).await?.into_inner())
    }

    pub async fn drop_non_linear_algorithm_index(
        &mut self,
        params: db_params::DropNonLinearAlgorithmIndexParams,
    ) -> Result<Del, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(DropNonLinearAlgorithmIndex {
            store: params.store.0,
            non_linear_indices: params
                .non_linear_indices
                .into_iter()
                .filter_map(|a| NonLinearAlgorithm::from(a).try_into().ok())
                .collect(),
            error_if_not_exists: params.error_if_not_exists,
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .drop_non_linear_algorithm_index(req)
            .await?
            .into_inner())
    }

    pub async fn del_key(&mut self, params: db_params::DelKeyParams) -> Result<Del, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(DelKey {
            store: params.store.0,
            keys: params
                .keys
                .into_iter()
                .map(|a| grpc_types::keyval::StoreKey { key: a.0 })
                .collect(),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.del_key(req).await?.into_inner())
    }

    pub async fn drop_store(
        &mut self,
        params: db_params::DropStoreParams,
    ) -> Result<Del, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(DropStore {
            store: params.store.0,
            error_if_not_exists: params.error_if_not_exists,
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.drop_store(req).await?.into_inner())
    }

    pub async fn del_pred(
        &mut self,
        params: db_params::DelPredParams,
    ) -> Result<Del, AhnlichError> {
        let tracing_id = params.tracing_id.clone();
        let mut req = tonic::Request::new(DelPred {
            store: params.store.0,
            condition: to_grpc_predicate_condition(params.condition).map(|a| *a),
        });
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.del_pred(req).await?.into_inner())
    }

    pub async fn info_server(
        &mut self,
        tracing_id: Option<String>,
    ) -> Result<ServerInfo, AhnlichError> {
        let mut req = tonic::Request::new(InfoServer {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self
            .client
            .info_server(req)
            .await?
            .into_inner()
            .info
            .expect("Server info should be Some"))
    }

    pub async fn list_stores(
        &mut self,
        tracing_id: Option<String>,
    ) -> Result<StoreList, AhnlichError> {
        let mut req = tonic::Request::new(ListStores {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.list_stores(req).await?.into_inner())
    }

    pub async fn list_clients(
        &mut self,
        tracing_id: Option<String>,
    ) -> Result<ClientList, AhnlichError> {
        let mut req = tonic::Request::new(ListClients {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.list_clients(req).await?.into_inner())
    }

    pub async fn ping(&mut self, tracing_id: Option<String>) -> Result<Pong, AhnlichError> {
        let mut req = tonic::Request::new(Ping {});
        add_trace_parent(&mut req, tracing_id);
        Ok(self.client.ping(req).await?.into_inner())
    }

    // Create list of instructions to execute in a pipeline loop
    // on the server end
    pub fn pipeline(&self) -> DbPipeline {
        DbPipeline {
            queries: vec![],
            client: self.client.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        time::Duration,
    };

    use super::*;
    use ahnlich_db::{cli::ServerConfig, server::handler::Server};
    use ahnlich_types::{
        db::ServerResponse,
        keyval::StoreKey,
        metadata::MetadataKey,
        predicate::{Predicate, PredicateCondition},
        similarity::Algorithm,
    };
    use grpc_types::{
        db::server::GetSimNEntry, metadata::metadata_value::Value, similarity::Similarity,
    };
    use once_cell::sync::Lazy;
    use utils::server::AhnlichServerUtils;

    static CONFIG: Lazy<ServerConfig> = Lazy::new(|| ServerConfig::default().os_select_port());

    #[tokio::test]
    async fn test_grpc_get_sim_n() {
        let server = Server::new(&CONFIG)
            .await
            .expect("Could not initialize server");
        let address = server.local_addr().expect("Could not get local addr");
        tokio::spawn(async move {
            // TODO: replace with server.start()
            server.task_manager().spawn_blocking(server).await;
        });
        // Allow some time for the server to start
        tokio::time::sleep(Duration::from_millis(100)).await;
        let mut db_client = DbClient::new(address.to_string())
            .await
            .expect("Could not initialize client");

        let create_store_params = db_params::CreateStoreParams::builder()
            .store("Main".to_string())
            .dimension(3)
            .create_predicates(HashSet::from_iter([MetadataKey::new("medal".into())]))
            .build();

        assert!(db_client.create_store(create_store_params).await.is_ok());

        let set_key_params = db_params::SetParams::builder()
            .store("Main".to_string())
            .inputs(vec![
                (
                    StoreKey(vec![1.2, 1.3, 1.4]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("silver".into()),
                    )]),
                ),
                (
                    StoreKey(vec![2.0, 2.1, 2.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("gold".into()),
                    )]),
                ),
                (
                    StoreKey(vec![5.0, 5.1, 5.2]),
                    HashMap::from_iter([(
                        MetadataKey::new("medal".into()),
                        MetadataValue::RawString("bronze".into()),
                    )]),
                ),
            ])
            .build();
        assert!(db_client.set(set_key_params).await.is_ok());
        // error due to dimension mismatch
        let get_sim_n_params = db_params::GetSimNParams::builder()
            .store("Main".to_string())
            .search_input(StoreKey(vec![1.1, 2.0]))
            .closest_n(2)
            .algorithm(Algorithm::EuclideanDistance)
            .build();
        assert!(db_client.get_sim_n(get_sim_n_params).await.is_err());

        let get_sim_n_params = db_params::GetSimNParams::builder()
            .store("Main".to_string())
            .search_input(StoreKey(vec![5.0, 2.1, 2.2]))
            .closest_n(2)
            .algorithm(Algorithm::CosineSimilarity)
            .condition(Some(PredicateCondition::Value(Predicate::Equals {
                key: MetadataKey::new("medal".into()),
                value: MetadataValue::RawString("gold".into()),
            })))
            .build();

        assert_eq!(
            db_client.get_sim_n(get_sim_n_params).await.unwrap(),
            GetSimNResult {
                entries: vec![GetSimNEntry {
                    key: Some(grpc_types::keyval::StoreKey {
                        key: vec![2.0, 2.1, 2.2]
                    }),
                    value: Some(grpc_types::keyval::StoreValue {
                        value: HashMap::from_iter([(
                            "medal".into(),
                            grpc_types::metadata::MetadataValue {
                                r#type: MetadataType::RawString.into(),
                                value: Some(Value::RawString("gold".into()))
                            },
                        )])
                    }),
                    similarity: Some(Similarity {
                        value: 0.9036338825194858
                    })
                }]
            }
        );
    }
}
