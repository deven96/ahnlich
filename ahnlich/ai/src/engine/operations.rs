use std::{collections::HashMap, sync::Arc};

use ahnlich_client_rs::db::DbClient;
use ahnlich_types::{
    ai::{
        models::AiModel,
        preprocess::PreprocessAction,
        query::{
            CreateNonLinearAlgorithmIndex, CreatePredIndex, CreateStore, DelKey,
            DelPred as AiDelPred, DropNonLinearAlgorithmIndex, DropPredIndex, DropStore,
            ListStores, PurgeStores, Set,
        },
        server::{AiStoreInfo, CreateIndex, Del, Set as SetResponse, StoreList, Unit},
    },
    db::{
        pipeline::{DbServerResponse, db_server_response::Response as DbResponse},
        query::{
            CreateNonLinearAlgorithmIndex as DbCreateNonLinearAlgorithmIndex,
            CreatePredIndex as DbCreatePredIndex, CreateStore as DbCreateStore, DelPred,
            DropNonLinearAlgorithmIndex as DbDropNonLinearAlgorithmIndex,
            DropPredIndex as DbDropPredIndex, DropStore as DbDropStore, Set as DbSetParams,
        },
        server::Set as DbSet,
    },
    keyval::{StoreInput, StoreName, StoreValue, store_input::Value},
    metadata::MetadataValue,
    predicates::{
        In, Predicate, PredicateCondition, predicate::Kind as PredicateKind,
        predicate_condition::Kind,
    },
    schema::Schema,
};
use itertools::Itertools;
use rayon::prelude::*;
use tonic::Status;

use crate::{
    AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY, AHNLICH_AI_RESERVED_META_KEY,
    cli::server::SupportedModels,
    engine::{ai::models::ModelDetails, store::AIStoreHandler},
    error::AIProxyError,
    manager::ModelManager,
};

#[allow(clippy::result_large_err)]
fn require_db_client(db_client: Option<Arc<DbClient>>) -> Result<Arc<DbClient>, Status> {
    db_client.ok_or_else(|| Status::failed_precondition("No DB client available"))
}

/// Extract original image dimensions from store inputs and inject into model_params.
/// This allows face detection models to normalize bounding boxes correctly by
/// accounting for letterboxing and aspect ratio transformations.
pub(crate) fn extract_and_inject_image_dimensions(
    inputs: &[StoreInput],
    model_params: &mut HashMap<String, String>,
) {
    for (idx, store_input) in inputs.iter().enumerate() {
        if let Some(Value::Image(image_bytes)) = &store_input.value
            && let Ok(img_array) =
                crate::engine::ai::models::ImageArray::try_from(image_bytes.as_slice())
        {
            let (width, height) = img_array.dimensions();
            model_params.insert(format!("orig_width_{}", idx), width.to_string());
            model_params.insert(format!("orig_height_{}", idx), height.to_string());
        }
    }
}

pub async fn create_store(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    params: CreateStore,
    parent_id: Option<String>,
) -> Result<Unit, Status> {
    let mut predicates = params.predicates;
    if params.store_original {
        predicates.push(AHNLICH_AI_RESERVED_META_KEY.to_string());
    }

    let index_model: AiModel = params
        .index_model
        .try_into()
        .map_err(|_| AIProxyError::InputNotSpecified("Index model".to_string()))?;
    let query_model: AiModel = params
        .query_model
        .try_into()
        .map_err(|_| AIProxyError::InputNotSpecified("Query model".to_string()))?;

    let model: ModelDetails = SupportedModels::from(&index_model).to_model_details();
    if model.is_one_to_many() {
        predicates.push(AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY.to_string());
    }

    require_db_client(db_client)?
        .create_store(
            DbCreateStore {
                store: params.store.clone(),
                dimension: model.embedding_size.get() as u32,
                create_predicates: predicates,
                non_linear_indices: params.non_linear_indices,
                error_if_exists: params.error_if_exists,
            },
            parent_id,
        )
        .await?;

    store_handler.create_store(
        StoreName {
            value: params.store,
        },
        Schema::default(),
        query_model,
        index_model,
        params.error_if_exists,
        params.store_original,
    )?;

    Ok(Unit {})
}

pub async fn create_pred_index(
    db_client: Option<Arc<DbClient>>,
    params: CreatePredIndex,
    parent_id: Option<String>,
) -> Result<CreateIndex, Status> {
    let res = require_db_client(db_client)?
        .create_pred_index(
            DbCreatePredIndex {
                store: params.store,
                predicates: params.predicates,
            },
            parent_id,
        )
        .await?;

    Ok(CreateIndex {
        created_indexes: res.created_indexes,
    })
}

pub async fn create_non_linear_algorithm_index(
    db_client: Option<Arc<DbClient>>,
    params: CreateNonLinearAlgorithmIndex,
    parent_id: Option<String>,
) -> Result<CreateIndex, Status> {
    let res = require_db_client(db_client)?
        .create_non_linear_algorithm_index(
            DbCreateNonLinearAlgorithmIndex {
                store: params.store,
                non_linear_indices: params.non_linear_indices,
            },
            parent_id,
        )
        .await?;

    Ok(CreateIndex {
        created_indexes: res.created_indexes,
    })
}

pub async fn set(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    model_manager: &ModelManager,
    params: Set,
    parent_id: Option<String>,
) -> Result<SetResponse, Status> {
    let mut model_params = params.model_params;
    let store_inputs: Vec<_> = params
        .inputs
        .iter()
        .filter_map(|input| input.key.clone())
        .collect();
    extract_and_inject_image_dimensions(&store_inputs, &mut model_params);

    let (db_inputs, delete_hashset) = store_handler
        .set(
            &StoreName {
                value: params.store.clone(),
            },
            &Schema::default(),
            params
                .inputs
                .into_par_iter()
                .flat_map(|a| {
                    a.key.map(|b| {
                        (
                            b,
                            StoreValue {
                                value: a.value.map(|val| val.value).unwrap_or_default(),
                            },
                        )
                    })
                })
                .collect(),
            model_manager,
            TryInto::<PreprocessAction>::try_into(params.preprocess_action)
                .map_err(AIProxyError::from)?,
            params.execution_provider.and_then(|a| a.try_into().ok()),
            model_params,
        )
        .await?;

    let mut pipeline = require_db_client(db_client)?.pipeline(parent_id);
    if let Some(del_hashset) = delete_hashset {
        pipeline.del_pred(DelPred {
            store: params.store.clone(),
            condition: Some(PredicateCondition {
                kind: Some(Kind::Value(Predicate {
                    kind: Some(PredicateKind::In(In {
                        key: AHNLICH_AI_RESERVED_META_KEY.to_string(),
                        values: del_hashset.into_iter().collect(),
                    })),
                })),
            }),
        });
    }

    pipeline.set(DbSetParams {
        store: params.store,
        inputs: db_inputs,
    });

    match pipeline.exec().await?.responses.as_slice() {
        [
            DbServerResponse {
                response: Some(DbResponse::Set(DbSet { upsert })),
            },
        ]
        | [
            DbServerResponse {
                response: Some(DbResponse::Del(_)),
            },
            DbServerResponse {
                response: Some(DbResponse::Set(DbSet { upsert })),
            },
        ] => Ok(SetResponse { upsert: *upsert }),
        responses => Err(AIProxyError::UnexpectedDBResponse(format!("{responses:?}")).into()),
    }
}

pub async fn drop_pred_index(
    db_client: Option<Arc<DbClient>>,
    mut params: DropPredIndex,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    params
        .predicates
        .retain(|val| val != AHNLICH_AI_RESERVED_META_KEY);

    let res = require_db_client(db_client)?
        .drop_pred_index(
            DbDropPredIndex {
                store: params.store,
                predicates: params.predicates,
                error_if_not_exists: params.error_if_not_exists,
            },
            parent_id,
        )
        .await?;

    Ok(Del {
        deleted_count: res.deleted_count,
    })
}

pub async fn drop_non_linear_algorithm_index(
    db_client: Option<Arc<DbClient>>,
    params: DropNonLinearAlgorithmIndex,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    let res = require_db_client(db_client)?
        .drop_non_linear_algorithm_index(
            DbDropNonLinearAlgorithmIndex {
                store: params.store,
                non_linear_indices: params.non_linear_indices,
                error_if_not_exists: params.error_if_not_exists,
            },
            parent_id,
        )
        .await?;

    Ok(Del {
        deleted_count: res.deleted_count,
    })
}

pub async fn del_key(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    params: DelKey,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    let store_original = store_handler.store_original(StoreName {
        value: params.store.clone(),
    }, &Schema::default())?;
    if !store_original {
        return Err(AIProxyError::DelKeyError.into());
    }

    let values = params
        .keys
        .into_iter()
        .map(|a| a.try_into())
        .collect::<Result<Vec<MetadataValue>, _>>()
        .map_err(|_| AIProxyError::InputNotSpecified("Store Input Value".to_string()))?;

    let res = require_db_client(db_client)?
        .del_pred(
            DelPred {
                store: params.store,
                condition: Some(PredicateCondition {
                    kind: Some(Kind::Value(Predicate {
                        kind: Some(PredicateKind::In(In {
                            key: AHNLICH_AI_RESERVED_META_KEY.to_string(),
                            values,
                        })),
                    })),
                }),
            },
            parent_id,
        )
        .await?;

    Ok(Del {
        deleted_count: res.deleted_count,
    })
}

pub async fn del_pred(
    db_client: Option<Arc<DbClient>>,
    params: AiDelPred,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    let res = require_db_client(db_client)?
        .del_pred(
            DelPred {
                store: params.store,
                condition: params.condition,
            },
            parent_id,
        )
        .await?;

    Ok(Del {
        deleted_count: res.deleted_count,
    })
}

pub async fn drop_store(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    params: DropStore,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    let store_name = StoreName {
        value: params.store,
    };

    if store_handler.get(&Schema::default(), &store_name).is_ok()
        && let Some(db_client) = db_client
    {
        db_client
            .drop_store(
                DbDropStore {
                    store: store_name.value.clone(),
                    error_if_not_exists: params.error_if_not_exists,
                },
                parent_id,
            )
            .await?;
    }

    let dropped = store_handler.drop_store(store_name, &Schema::default(), params.error_if_not_exists)?;
    Ok(Del {
        deleted_count: dropped as u64,
    })
}

pub async fn purge_stores(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    _params: PurgeStores,
    parent_id: Option<String>,
) -> Result<Del, Status> {
    let store_names: Vec<StoreName> = store_handler
        .list_stores()
        .into_iter()
        .map(|store| StoreName { value: store.name })
        .collect();

    if let Some(db_client) = db_client {
        for store_name in &store_names {
            db_client
                .drop_store(
                    DbDropStore {
                        store: store_name.value.clone(),
                        error_if_not_exists: false,
                    },
                    parent_id.clone(),
                )
                .await?;
        }
    }

    Ok(Del {
        deleted_count: store_handler.purge_stores() as u64,
    })
}

pub async fn list_stores(
    store_handler: &AIStoreHandler,
    db_client: Option<Arc<DbClient>>,
    _params: ListStores,
    parent_id: Option<String>,
) -> Result<StoreList, Status> {
    let mut stores: Vec<AiStoreInfo> = store_handler.list_stores().into_iter().sorted().collect();

    if let Some(db_client) = db_client
        && let Ok(db_store_list) = db_client.list_stores(parent_id).await
    {
        let db_map: HashMap<String, ahnlich_types::db::server::StoreInfo> = db_store_list
            .stores
            .into_iter()
            .map(|s| (s.name.clone(), s))
            .collect();
        for store in &mut stores {
            if let Some(db_store_info) = db_map.get(&store.name) {
                store.predicate_indices = db_store_info
                    .predicate_indices
                    .iter()
                    .filter(|p| {
                        *p != AHNLICH_AI_RESERVED_META_KEY
                            && *p != AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY
                    })
                    .cloned()
                    .collect();
                store.db_info = Some(db_store_info.clone());
            }
        }
    }

    Ok(StoreList { stores })
}
