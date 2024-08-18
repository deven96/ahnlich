use crate::cli::server::SupportedModels;
use crate::engine::ai::models::Model;
use crate::error::AIProxyError;
use crate::AHNLICH_AI_RESERVED_META_KEY;
use ahnlich_types::ai::{
    AIModel, AIStoreInfo, AIStoreInputType, ImageAction, PreprocessAction, StringAction,
};
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::metadata::MetadataValue;
use flurry::HashMap as ConcurrentHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Contains all the stores that have been created in memory
#[derive(Debug)]
pub struct AIStoreHandler {
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    stores: AIStores,
    pub write_flag: Arc<AtomicBool>,
    supported_models: Vec<SupportedModels>,
}

pub type AIStores = Arc<ConcurrentHashMap<StoreName, Arc<AIStore>>>;

impl AIStoreHandler {
    pub fn new(write_flag: Arc<AtomicBool>, supported_models: Vec<SupportedModels>) -> Self {
        Self {
            stores: Arc::new(ConcurrentHashMap::new()),
            write_flag,
            supported_models,
        }
    }
    pub(crate) fn get_stores(&self) -> AIStores {
        self.stores.clone()
    }

    #[cfg(test)]
    pub fn write_flag(&self) -> Arc<AtomicBool> {
        self.write_flag.clone()
    }

    fn set_write_flag(&self) {
        let _ = self
            .write_flag
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst);
    }

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
                    query_model: store.query_model.clone(),
                    index_model: store.index_model.clone(),
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
        store_name: &StoreName,
        store_input: StoreInput,
        store_value: StoreValue,
        preprocess_action: &PreprocessAction,
    ) -> Result<(StoreKey, StoreValue), AIProxyError> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        if store_value.contains_key(metadata_key) {
            return Err(AIProxyError::ReservedError(metadata_key.to_string()));
        }
        let store = self.get(store_name)?;

        let store_input_type: AIStoreInputType = (&store_input).into();
        let index_model_repr: Model = (&store.index_model).into();

        if store_input_type.to_string() != index_model_repr.input_type() {
            return Err(AIProxyError::StoreSetTypeMismatchError {
                index_model_type: index_model_repr.input_type(),
                storeinput_type: store_input_type.to_string(),
            });
        }

        let metadata_value: MetadataValue = store_input.clone().into();
        let mut final_store_value: StdHashMap<MetadataKey, MetadataValue> =
            store_value.clone().into_iter().collect();
        final_store_value.insert(metadata_key.clone(), metadata_value);

        let store_key =
            self.create_store_key(store_input, &store.index_model, preprocess_action)?;
        return Ok((store_key, final_store_value));
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

        let model: Model = index_store_model.into();
        let store_key = model.model_ndarray(&processed_input);
        Ok(store_key)
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
        let model: Model = index_model.into();
        if let Some(max_token_size) = model.max_input_token() {
            if input.len() > max_token_size.into() {
                if let StringAction::ErrorIfTokensExceed = string_action {
                    return Err(AIProxyError::TokenExceededError {
                        input_token_size: input.len(),
                        max_token_size: max_token_size.into(),
                    });
                } else {
                    // truncate raw string
                    // let tokenized_input;
                    let _input = input.as_str()[..max_token_size.into()].to_string();
                }
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
        let model: Model = index_model.into();

        if let Some(max_image_dimensions) = model.max_image_dimensions() {
            if input.len() > max_image_dimensions.into() {
                if let ImageAction::ErrorIfDimensionsMismatch = image_action {
                    return Err(AIProxyError::ImageDimensionsMismatchError {
                        image_dimensions: input.len(),
                        max_dimensions: max_image_dimensions.into(),
                    });
                } else {
                    // resize image
                }
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
