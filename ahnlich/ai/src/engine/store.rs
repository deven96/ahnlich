use crate::cli::server::SupportedModels;
use crate::engine::ai::models::Model;
use crate::error::AIProxyError;
use crate::manager::ModelManager;
use crate::AHNLICH_AI_RESERVED_META_KEY;
use ahnlich_types::ai::{AIModel, AIStoreInfo, AIStoreInputType, PreprocessAction};
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataValue;
use fallible_collections::FallibleVec;
use flurry::HashMap as ConcurrentHashMap;
use futures::stream::FuturesOrdered;
use futures::StreamExt;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet as StdHashSet;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use utils::persistence::AhnlichPersistenceUtils;

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub struct AIStoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: AIStores,
    pub write_flag: Arc<AtomicBool>,
    supported_models: Vec<SupportedModels>,
}

pub type AIStores = Arc<ConcurrentHashMap<StoreName, Arc<AIStore>>>;

type StoreSetResponse = (Vec<(StoreKey, StoreValue)>, StdHashSet<MetadataValue>);
type StoreValidateResponse = (Vec<(StoreInput, StoreValue)>, StdHashSet<MetadataValue>);
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
        query_model: AIModel,
        index_model: AIModel,
    ) -> Result<(), AIProxyError> {
        if !self.supported_models.contains(&(&query_model).into())
            || !self.supported_models.contains(&(&index_model).into())
        {
            return Err(AIProxyError::AIModelNotInitialized);
        }

        let index_model_repr: Model = (&index_model).into();
        let query_model_repr: Model = (&query_model).into();

        if index_model_repr.embedding_size() != query_model_repr.embedding_size() {
            return Err(AIProxyError::DimensionsMismatchError {
                index_model_dim: index_model_repr.embedding_size().into(),
                query_model_dim: query_model_repr.embedding_size().into(),
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
                )),
                &self.stores.guard(),
            )
            .is_err()
        {
            return Err(AIProxyError::StoreAlreadyExists(store_name.clone()));
        }
        self.set_write_flag();
        Ok(())
    }

    /// matches LISTSTORES - to return statistics of all stores
    #[tracing::instrument(skip(self))]
    pub(crate) fn list_stores(&self) -> StdHashSet<AIStoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| {
                let model: Model = (&store.index_model).into();

                AIStoreInfo {
                    name: store_name.clone(),
                    query_model: store.query_model,
                    index_model: store.index_model,
                    embedding_size: model.embedding_size().into(),
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
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;

        let metadata_value: MetadataValue = store_input.clone().into();
        store_value.insert(metadata_key.clone(), metadata_value);
        return Ok((store_input, store_value));
    }

    /// Validates storeinputs against a store and checks storevalue for reservedkey.
    #[tracing::instrument(skip(self, inputs), fields(input_length=inputs.len()))]
    pub(crate) async fn validate_and_prepare_store_data(
        &self,
        store_name: &StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
    ) -> Result<StoreValidateResponse, AIProxyError> {
        let store = self.get(store_name)?;
        let mut output: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let mut delete_hashset = StdHashSet::with_capacity(inputs.len());
        let mut handles = inputs
            .into_iter()
            .map(|(store_input, store_value)| {
                let temp_store = store.clone();
                tokio::spawn(async move {
                    Self::process_store_inputs(temp_store, (store_input, store_value)).await
                })
            })
            .collect::<FuturesOrdered<_>>();

        while let Some(value) = handles.next().await {
            match value {
                Ok(Ok((store_input, store_value, del_entry))) => {
                    output.push((store_input, store_value));
                    delete_hashset.insert(del_entry);
                }
                Ok(Err(err)) => return Err(err),
                Err(join_error) => return Err(AIProxyError::StandardError(join_error.to_string())),
            }
        }
        Ok((output, delete_hashset))
    }

    #[tracing::instrument(skip(store, input))]
    pub(crate) async fn process_store_inputs(
        store: Arc<AIStore>,
        input: (StoreInput, StoreValue),
    ) -> Result<(StoreInput, StoreValue, MetadataValue), AIProxyError> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        let (store_input, mut store_value) = input;
        if store_value.contains_key(metadata_key) {
            return Err(AIProxyError::ReservedError(metadata_key.to_string()));
        }
        let store_input_type: AIStoreInputType = (&store_input).into();
        let index_model_repr: Model = (&store.index_model).into();
        if store_input_type.to_string() != index_model_repr.input_type() {
            return Err(AIProxyError::StoreSetTypeMismatchError {
                index_model_type: index_model_repr.input_type(),
                storeinput_type: store_input_type.to_string(),
            });
        }
        let metadata_value: MetadataValue = store_input.clone().into();
        store_value.insert(metadata_key.clone(), metadata_value.clone());
        Ok((store_input, store_value, metadata_value))
    }

    /// Stores storeinput into ahnlich db
    #[tracing::instrument(skip(self), fields(input_length=inputs.len()))]
    pub(crate) async fn set(
        &self,
        store_name: &StoreName,
        inputs: Vec<(StoreInput, StoreValue)>,
        model_manager: &ModelManager,
        preprocess_action: PreprocessAction,
    ) -> Result<StoreSetResponse, AIProxyError> {
        let store = self.get(store_name)?;
        let (validated_data, delete_hashset) = self
            .validate_and_prepare_store_data(store_name, inputs)
            .await?;

        let (store_inputs, store_values): (Vec<_>, Vec<_>) = validated_data.into_iter().unzip();
        let store_keys = model_manager
            .handle_request(&store.index_model, store_inputs, preprocess_action)
            .await?;

        let output = std::iter::zip(store_keys.into_iter(), store_values.into_iter()).collect();
        Ok((output, delete_hashset))
    }

    /// Converts (storekey, storevalue) into (storeinput, storevalue)
    /// by removing the reserved_key from storevalue
    #[tracing::instrument(skip(self))]
    pub(crate) fn store_key_val_to_store_input_val(
        &self,
        output: Vec<(StoreKey, StoreValue)>,
    ) -> Vec<(StoreInput, StoreValue)> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;

        // TODO: Will parallelized
        output
            .into_iter()
            .filter_map(|(_, mut store_value)| {
                store_value
                    .remove(metadata_key)
                    .map(|val| (val, store_value))
            })
            .map(|(metadata_value, store_value)| {
                let store_input: StoreInput = metadata_value.into();
                (store_input, store_value)
            })
            .collect()
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn get_ndarray_repr_for_store(
        &self,
        store_name: &StoreName,
        store_input: &StoreInput,
    ) -> Result<StoreKey, AIProxyError> {
        let store = self.get(store_name)?;
        let model: Model = (&store.index_model).into();
        Ok(model.model_ndarray(store_input))
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
    query_model: AIModel,
    index_model: AIModel,
}

impl AIStore {
    pub(super) fn create(
        store_name: StoreName,
        query_model: AIModel,
        index_model: AIModel,
    ) -> Self {
        Self {
            name: store_name,
            query_model,
            index_model,
        }
    }
}
