use crate::cli::server::SupportedModels;
use crate::engine::ai::models::InputAction;
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
use rayon::iter::{IndexedParallelIterator, IntoParallelIterator, ParallelIterator};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet as StdHashSet;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use utils::parallel;
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

type StoreSetResponse = (
    Vec<(StoreKey, StoreValue)>,
    Option<StdHashSet<MetadataValue>>,
);
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
        query_model: AIModel,
        index_model: AIModel,
        error_if_exists: bool,
        store_original: bool,
    ) -> Result<(), AIProxyError> {
        if !self.supported_models.contains(&(&query_model).into())
            || !self.supported_models.contains(&(&index_model).into())
        {
            return Err(AIProxyError::AIModelNotInitialized);
        }

        let index_model_repr: Model = (&index_model).into();
        let query_model_repr: Model = (&query_model).into();

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
    pub(crate) fn list_stores(&self) -> StdHashSet<AIStoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| {
                let model: Model = (&store.index_model).into();

                AIStoreInfo {
                    name: store_name.clone(),
                    query_model: store.query_model,
                    index_model: store.index_model,
                    embedding_size: model.embedding_size.into(),
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
        index_model: AIModel,
        inputs: Vec<(StoreInput, StoreValue)>,
        store_original: bool,
    ) -> Result<StoreValidateResponse, AIProxyError> {
        let mut output: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let mut delete_hashset = StdHashSet::new();
        for (store_input, mut store_value) in inputs {
            let store_input_type: AIStoreInputType = (&store_input).into();
            let index_model_repr: Model = (&index_model).into();
            if store_input_type != index_model_repr.input_type() {
                return Err(AIProxyError::StoreTypeMismatchError {
                    action: InputAction::Index,
                    index_model_type: index_model_repr.input_type(),
                    storeinput_type: store_input_type,
                });
            }
            if store_original {
                let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
                if store_value.contains_key(metadata_key) {
                    return Err(AIProxyError::ReservedError(metadata_key.to_string()));
                }
                let metadata_value: MetadataValue = store_input.clone().into();
                store_value.insert(metadata_key.clone(), metadata_value.clone());
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
    ) -> Result<StoreSetResponse, AIProxyError> {
        let store = self.get(store_name)?;
        let (validated_data, delete_hashset) =
            self.validate_and_prepare_store_data(store_name, inputs)?;

        let (store_inputs, store_values): (Vec<_>, Vec<_>) = validated_data.into_iter().unzip();
        let store_keys = model_manager
            .handle_request(
                &store.index_model,
                store_inputs,
                preprocess_action,
                InputAction::Index,
            )
            .await?;

        let output = std::iter::zip(store_keys.into_iter(), store_values.into_iter()).collect();
        Ok((output, delete_hashset))
    }

    /// Converts storeinput into a tuple of storekey and storevalue.
    /// Fails if the store input type does not match the store index_type
    #[tracing::instrument(skip(self))]
    pub(crate) fn create_store_key(
        &self,
        store_input: StoreInput,
        index_store_model: &AIModel,
        preprocess_action: &PreprocessAction,
    ) -> Result<StoreKey, AIProxyError> {
        // Process the inner value of a store input and convert it into a ndarray by passing
        // it into  index model. Create a storekey from ndarray
        let processed_input =
            self.preprocess_store_input(preprocess_action, store_input, index_store_model)?;
        let store_key = index_store_model.model_ndarray(&processed_input);
        Ok(store_key)
    }

    /// Converts (storekey, storevalue) into (storeinput, storevalue)
    /// by removing the reserved_key from storevalue
    #[tracing::instrument(skip(self, output), fields(output_len=output.len()))]
    pub(crate) fn store_key_val_to_store_input_val(
        &self,
        output: Vec<(StoreKey, StoreValue)>,
    ) -> Vec<(Option<StoreInput>, StoreValue)> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;

        output
            .into_par_iter()
            .map(|(_, mut store_value)| {
                let store_input = store_value.remove(metadata_key).map(|val| val.into());
                (store_input, store_value)
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
    ) -> Result<StoreKey, AIProxyError> {
        let store = self.get(store_name)?;
        let mut store_keys = model_manager
            .handle_request(
                &store.index_model,
                vec![store_input],
                preprocess_action,
                InputAction::Query,
            )
            .await?;

        Ok(store_keys.pop().expect("Expected an embedding value."))
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

    #[tracing::instrument(skip(self))]
    pub(crate) fn preprocess_store_input(
        &self,
        process_action: &PreprocessAction,
        input: StoreInput,
        index_model: &AIModel,
    ) -> Result<StoreInput, AIProxyError> {
        match (process_action, input) {
            (PreprocessAction::Image(image_action), StoreInput::Image(image_input)) => {
                // resize image and edit
                let output = self.process_image(image_input, index_model, image_action)?;
                Ok(output)
            }
            (PreprocessAction::RawString(string_action), StoreInput::RawString(string_input)) => {
                let output =
                    self.preprocess_raw_string(string_input, index_model, string_action)?;
                Ok(output)
            }
            (PreprocessAction::RawString(_), StoreInput::Image(_)) => {
                Err(AIProxyError::PreprocessingMismatchError {
                    input_type: AIStoreInputType::Image,
                    preprocess_action: process_action.clone(),
                })
            }

            (PreprocessAction::Image(_), StoreInput::RawString(_)) => {
                Err(AIProxyError::PreprocessingMismatchError {
                    input_type: AIStoreInputType::RawString,
                    preprocess_action: process_action.clone(),
                })
            }
        }
    }
    fn preprocess_raw_string(
        &self,
        input: String,
        index_model: &AIModel,
        string_action: &StringAction,
    ) -> Result<StoreInput, AIProxyError> {
        // tokenize string, return error if max token
        //let tokenized_input;
        let model_embedding_dim = index_model.model_info().embedding_size;
        if input.len() > model_embedding_dim.into() {
            if let StringAction::ErrorIfTokensExceed = string_action {
                return Err(AIProxyError::TokenExceededError {
                    input_token_size: input.len(),
                    model_embedding_size: model_embedding_dim.into(),
                });
            } else {
                // truncate raw string
                // let tokenized_input;
                let _input = input.as_str()[..model_embedding_dim.into()].to_string();
            }
        }
        Ok(StoreInput::RawString(input))
    }

    fn process_image(
        &self,
        input: Vec<u8>,
        index_model: &AIModel,
        image_action: &ImageAction,
    ) -> Result<StoreInput, AIProxyError> {
        // process image, return error if max dimensions exceeded
        // let image_data;
        let model_embedding_dim = index_model.embedding_size().into();
        if input.len() > model_embedding_dim {
            if let ImageAction::ErrorIfDimensionsMismatch = image_action {
                return Err(AIProxyError::ImageDimensionsMismatchError {
                    image_dimensions: input.len(),
                    max_dimensions: model_embedding_dim,
                });
            } else {
                // resize image
            }
        }
        Ok(StoreInput::Image(input))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIStore {
    name: StoreName,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    query_model: AIModel,
    index_model: AIModel,
    store_original: bool,
}

impl AIStore {
    pub(super) fn create(
        store_name: StoreName,
        query_model: AIModel,
        index_model: AIModel,
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
