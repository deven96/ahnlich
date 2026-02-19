use crate::AHNLICH_AI_RESERVED_META_KEY;
use crate::cli::server::SupportedModels;
use crate::engine::ai::models::InputAction;
use crate::error::AIProxyError;
use crate::manager::ModelManager;
use ahnlich_types::ai::execution_provider::ExecutionProvider;
use ahnlich_types::ai::models::AiModel;
use ahnlich_types::ai::models::AiStoreInputType;
use ahnlich_types::ai::preprocess::PreprocessAction;
use ahnlich_types::ai::server::AiStoreInfo;
use ahnlich_types::ai::server::GetEntry;
use ahnlich_types::keyval::DbStoreEntry;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::metadata::metadata_value;
use fallible_collections::FallibleVec;
use papaya::HashMap as ConcurrentHashMap;
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet as StdHashSet;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use utils::parallel;
use utils::persistence::AhnlichPersistenceUtils;

use super::ai::models::ModelDetails;
use super::ai::models::ModelResponse;

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub struct AIStoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: AIStores,
    pub write_flag: Arc<AtomicBool>,
    supported_models: Vec<SupportedModels>,
}

pub type AIStores = Arc<ConcurrentHashMap<StoreName, Arc<AIStore>>>;

type StoreSetResponse = (Vec<DbStoreEntry>, Option<StdHashSet<MetadataValue>>);
type StoreValidateResponse = (
    Vec<(StoreInput, StoreValue)>,
    Option<StdHashSet<MetadataValue>>,
);
impl AhnlichPersistenceUtils for AIStoreHandler {
    type PersistenceObject = AIStores;

    #[tracing::instrument(skip_all)]
    fn write_flag(&self) -> Arc<AtomicBool> {
        self.write_flag.clone()
    }

    #[tracing::instrument(skip(self))]
    fn get_snapshot(&self) -> Self::PersistenceObject {
        self.stores.clone()
    }
}

impl AIStoreHandler {
    pub fn new(write_flag: Arc<AtomicBool>, supported_models: Vec<SupportedModels>) -> Self {
        Self {
            stores: Arc::new(ConcurrentHashMap::new()),
            write_flag,
            supported_models,
        }
    }

    #[tracing::instrument(skip(self))]
    fn set_write_flag(&self) {
        let _ = self
            .write_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn use_snapshot(&mut self, stores_snapshot: AIStores) {
        self.stores = stores_snapshot;
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn create_store(
        &self,
        store_name: StoreName,
        query_model: AiModel,
        index_model: AiModel,
        error_if_exists: bool,
        store_original: bool,
    ) -> Result<(), AIProxyError> {
        if !self.supported_models.contains(&(&query_model).into())
            || !self.supported_models.contains(&(&index_model).into())
        {
            return Err(AIProxyError::AIModelNotInitialized);
        }

        let index_model_repr: ModelDetails = SupportedModels::from(&index_model).to_model_details();
        let query_model_repr: ModelDetails = SupportedModels::from(&query_model).to_model_details();

        if index_model_repr.embedding_size != query_model_repr.embedding_size {
            return Err(AIProxyError::DimensionsMismatchError {
                index_model_dim: index_model_repr.embedding_size.into(),
                query_model_dim: query_model_repr.embedding_size.into(),
            });
        }

        if self
            .stores
            .try_insert(
                store_name.clone(),
                Arc::new(AIStore::create(
                    store_name.clone(),
                    query_model,
                    index_model,
                    store_original,
                )),
                &self.stores.guard(),
            )
            .is_err()
            && error_if_exists
        {
            return Err(AIProxyError::StoreAlreadyExists(store_name.clone()));
        }
        self.set_write_flag();
        Ok(())
    }

    /// matches LISTSTORES - to return statistics of all stores
    #[tracing::instrument(skip(self))]
    pub(crate) fn list_stores(&self) -> StdHashSet<AiStoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| {
                let model: ModelDetails =
                    SupportedModels::from(&store.index_model).to_model_details();

                AiStoreInfo {
                    name: store_name.value.clone(),
                    query_model: store.query_model.into(),
                    index_model: store.index_model.into(),
                    embedding_size: model.embedding_size.get() as u64,
                }
            })
            .collect()
    }

    /// Returns a store using the store name, else returns an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn get(&self, store_name: &StoreName) -> Result<Arc<AIStore>, AIProxyError> {
        let store = self
            .stores
            .get(store_name, &self.stores.guard())
            .cloned()
            .ok_or(AIProxyError::StoreNotFound(store_name.clone()))?;
        Ok(store)
    }

    /// Converts storeinput into a tuple of storekey and storevalue.
    /// Fails if the store input type does not match the store index_type
    #[tracing::instrument(skip(self))]
    pub(crate) fn store_input_to_store_key_val(
        &self,
        store_input: StoreInput,
        mut store_value: StoreValue,
        preprocess_action: &PreprocessAction,
    ) -> Result<(StoreInput, StoreValue), AIProxyError> {
        let metadata_value: MetadataValue = store_input
            .clone()
            .try_into()
            .map_err(|_| AIProxyError::InputNotSpecified("Store Input Value".to_string()))?;

        store_value
            .value
            .insert(AHNLICH_AI_RESERVED_META_KEY.to_string(), metadata_value);
        return Ok((store_input, store_value));
    }

    /// Validates storeinputs against a store and checks storevalue for reservedkey.
    #[tracing::instrument(skip(self, inputs), fields(input_length=inputs.len(), num_threads = rayon::current_num_threads()))]
    pub(crate) fn validate_and_prepare_store_data(
        &self,
        store_name: &StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
    ) -> Result<StoreValidateResponse, AIProxyError> {
        let store = self.get(store_name)?;
        let index_model = store.index_model;
        let chunk_size = parallel::chunk_size(inputs.len());
        inputs
            .into_par_iter()
            .chunks(chunk_size)
            .map(|input| Self::preprocess_store_input(index_model, input, store.store_original))
            .try_reduce(
                || (Vec::new(), None),
                |(mut acc_vec, mut acc_set), chunk_res| {
                    let (chunk_vec, chunk_set) = chunk_res;
                    acc_vec.extend(chunk_vec);
                    if let (Some(acc), Some(chunk)) = (&mut acc_set, chunk_set) {
                        acc.extend(chunk)
                    }
                    Ok((acc_vec, acc_set))
                },
            )
    }

    #[tracing::instrument(skip(inputs))]
    pub(crate) fn preprocess_store_input(
        index_model: AiModel,
        inputs: Vec<(StoreInput, StoreValue)>,
        store_original: bool,
    ) -> Result<StoreValidateResponse, AIProxyError> {
        let mut output: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let mut delete_hashset = StdHashSet::new();
        for (store_input, mut store_value) in inputs {
            let store_input_type: AiStoreInputType = (&store_input)
                .try_into()
                .map_err(|_| AIProxyError::InputNotSpecified("Store Input Value".to_string()))?;
            let index_model_repr: ModelDetails =
                SupportedModels::from(&index_model).to_model_details();
            if store_input_type != index_model_repr.input_type() {
                return Err(AIProxyError::StoreTypeMismatchError {
                    action: InputAction::Index,
                    index_model_type: index_model_repr.input_type(),
                    storeinput_type: store_input_type,
                });
            }
            if store_original {
                if store_value.value.contains_key(AHNLICH_AI_RESERVED_META_KEY) {
                    return Err(AIProxyError::ReservedError(
                        AHNLICH_AI_RESERVED_META_KEY.to_string(),
                    ));
                }

                let metadata_value: MetadataValue =
                    store_input.clone().try_into().map_err(|_| {
                        AIProxyError::InputNotSpecified("Store Input Value".to_string())
                    })?;

                store_value.value.insert(
                    AHNLICH_AI_RESERVED_META_KEY.to_string(),
                    metadata_value.clone(),
                );
                delete_hashset.insert(metadata_value);
            }
            output.try_push((store_input, store_value))?;
        }
        let delete_hashset = (store_original).then_some(delete_hashset);
        Ok((output, delete_hashset))
    }

    /// Stores storeinput into ahnlich db
    #[tracing::instrument(skip(self, inputs), fields(input_length=inputs.len()))]
    pub(crate) async fn set(
        &self,
        store_name: &StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
        model_manager: &ModelManager,
        preprocess_action: PreprocessAction,
        execution_provider: Option<ExecutionProvider>,
    ) -> Result<StoreSetResponse, AIProxyError> {
        let store = self.get(store_name)?;
        if inputs.is_empty() {
            return Ok((vec![], None));
        }
        let (validated_data, delete_hashset) =
            self.validate_and_prepare_store_data(store_name, inputs)?;

        let (store_inputs, store_values): (Vec<_>, Vec<_>) = validated_data.into_iter().unzip();
        let store_keys = model_manager
            .handle_request(
                &store.index_model,
                store_inputs,
                preprocess_action,
                InputAction::Index,
                execution_provider,
            )
            .await?;

        let output = std::iter::zip(store_keys.into_iter(), store_values.into_iter())
            .flat_map(|(k, v)| match k {
                ModelResponse::OneToOne(key) => vec![DbStoreEntry {
                    key: Some(key),
                    value: Some(v),
                }],
                ModelResponse::OneToMany(keys) => {
                    // For OneToMany models (e.g., Buffalo_L face recognition), add sequential
                    // index metadata to each output for identification and deletion
                    let model_details =
                        SupportedModels::from(&store.index_model).to_model_details();
                    let is_one_to_many = model_details.is_one_to_many();
                    keys.into_iter()
                        .enumerate()
                        .map(|(output_idx, single_key)| {
                            let mut value = v.clone();
                            if is_one_to_many {
                                // Add index as metadata for OneToMany models
                                value.value.insert(
                                    crate::AHNLICH_AI_ONE_TO_MANY_INDEX_META_KEY.to_string(),
                                    MetadataValue {
                                        value: Some(metadata_value::Value::RawString(
                                            output_idx.to_string(),
                                        )),
                                    },
                                );
                            }
                            DbStoreEntry {
                                key: Some(single_key),
                                value: Some(value),
                            }
                        })
                        .collect()
                }
            })
            .collect();
        Ok((output, delete_hashset))
    }

    #[tracing::instrument(skip(self, input), fields(input_len=input.len()))]
    pub(crate) fn db_store_entry_to_store_get_key(
        &self,
        input: Vec<DbStoreEntry>,
    ) -> Vec<GetEntry> {
        input
            .into_par_iter()
            .flat_map(|store_entry| {
                if let Some(mut value) = store_entry.value {
                    let key = value
                        .value
                        .remove(AHNLICH_AI_RESERVED_META_KEY)
                        .and_then(|val| StoreInput::try_from(val).ok());

                    return Some(GetEntry {
                        key,
                        value: Some(value),
                    });
                }
                None
            })
            .collect()
    }

    /// Converts (storekey, storevalue) into (storeinput, storevalue)
    /// by removing the reserved_key from storevalue
    #[tracing::instrument(skip(self, output), fields(output_len=output.len()))]
    pub(crate) fn store_key_val_to_store_input_val(
        &self,
        output: Vec<(StoreKey, StoreValue)>,
    ) -> Vec<(Option<StoreInput>, StoreValue)> {
        output
            .into_par_iter()
            .map(|(_, mut store_value)| {
                // NOTE: verify the logic here
                let store_input = store_value
                    .value
                    .remove(AHNLICH_AI_RESERVED_META_KEY)
                    .map(|val| val.try_into().ok());

                match store_input {
                    Some(entry) => (entry, store_value),
                    None => (None, store_value),
                }
            })
            .collect()
    }

    #[tracing::instrument(skip(self))]
    pub(crate) async fn get_ndarray_repr_for_store(
        &self,
        store_name: &StoreName,
        store_input: StoreInput,
        model_manager: &ModelManager,
        preprocess_action: PreprocessAction,
        execution_provider: Option<ExecutionProvider>,
    ) -> Result<StoreKey, AIProxyError> {
        let store = self.get(store_name)?;
        let mut store_keys = model_manager
            .handle_request(
                &store.query_model,
                vec![store_input],
                preprocess_action,
                InputAction::Query,
                execution_provider,
            )
            .await?;

        match store_keys.pop() {
            Some(first_store_key) => match first_store_key {
                ModelResponse::OneToOne(resp) => Ok(resp),
                ModelResponse::OneToMany(mut many) => match many.len() {
                    0 => Err(AIProxyError::ModelInputToEmbeddingError),
                    1 => Ok(many.pop().unwrap()),
                    n => Err(AIProxyError::MultipleEmbeddingsForQuery(n)),
                },
            },
            None => Err(AIProxyError::ModelInputToEmbeddingError),
        }
    }

    /// Matches DROPSTORE - Drops a store if exist, else returns an error
    #[tracing::instrument(skip(self))]
    pub(crate) fn drop_store(
        &self,
        store_name: StoreName,
        error_if_not_exists: bool,
    ) -> Result<usize, AIProxyError> {
        let pinned = self.stores.pin();
        let removed = pinned.remove(&store_name).is_some();
        if !removed && error_if_not_exists {
            return Err(AIProxyError::StoreNotFound(store_name));
        }
        let removed = if !removed {
            0
        } else {
            self.set_write_flag();
            1
        };
        Ok(removed)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn store_original(&self, store_name: StoreName) -> Result<bool, AIProxyError> {
        let store = self.get(&store_name)?;
        Ok(store.store_original)
    }

    /// Matches DestroyDatabase - Drops all the stores in the database
    #[tracing::instrument(skip(self))]
    pub(crate) fn purge_stores(&self) -> usize {
        let store_length = self.stores.pin().len();
        let guard = self.stores.guard();
        self.stores.clear(&guard);
        store_length
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIStore {
    name: StoreName,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    query_model: AiModel,
    index_model: AiModel,
    store_original: bool,
}

impl AIStore {
    pub(super) fn create(
        store_name: StoreName,
        query_model: AiModel,
        index_model: AiModel,
        store_original: bool,
    ) -> Self {
        Self {
            name: store_name,
            query_model,
            index_model,
            store_original,
        }
    }
}
