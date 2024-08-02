use crate::error::AIProxyError;
use crate::AHNLICH_AI_RESERVED_META_KEY;
use ahnlich_types::ai::{
    AIModel, AIStoreInfo, AIStoreInputTypes, ImageAction, PreprocessAction, StringAction,
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
}

pub type AIStores = Arc<ConcurrentHashMap<StoreName, Arc<AIStore>>>;

impl AIStoreHandler {
    pub fn new(write_flag: Arc<AtomicBool>) -> Self {
        Self {
            stores: Arc::new(ConcurrentHashMap::new()),
            write_flag,
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
        if self
            .stores
            .try_insert(
                store_name.clone(),
                Arc::new(AIStore::create(
                    store_name.clone(),
                    query_model.clone(),
                    index_model.clone(),
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
            .map(|(store_name, store)| AIStoreInfo {
                name: store_name.clone(),
                query_model: store.query_model.clone(),
                index_model: store.index_model.clone(),
                embedding_size: store.index_model.embedding_size().into(),
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
        store_value: &StoreValue,
        preprocess_action: &PreprocessAction,
    ) -> Result<(StoreKey, StoreValue), AIProxyError> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        if store_value.contains_key(metadata_key) {
            return Err(AIProxyError::ReservedError(metadata_key.to_string()));
        }
        let store = self.get(store_name)?;

        let store_input_type: AIStoreInputTypes = store_input.clone().into();
        let store_index_model_info = store.index_model.model_info();

        if store_input_type != store_index_model_info.input_type {
            return Err(AIProxyError::StoreSetTypeMismatchError {
                store_index_model_type: store_index_model_info.input_type,
                storeinput_type: store_input_type,
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
        let mut store_input = store_input;

        self.process_store_input(preprocess_action, &mut store_input, index_store_model)?;
        let store_key = index_store_model.model_ndarray(&store_input);
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
        Ok(store.index_model.model_ndarray(store_input))
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
    pub(crate) fn process_store_input(
        &self,
        process_action: &PreprocessAction,
        input: &mut StoreInput,
        index_model: &AIModel,
    ) -> Result<(), AIProxyError> {
        match process_action {
            PreprocessAction::Image(image_action) => {
                // resize image and edit
                self.process_image(input, index_model, image_action)?;
                Ok(())
            }
            PreprocessAction::RawString(string_action) => {
                self.preprocess_raw_string(input, index_model, string_action)?;
                Ok(())
            }
        }
    }
    fn preprocess_raw_string(
        &self,
        input: &mut StoreInput,
        index_model: &AIModel,
        string_action: &StringAction,
    ) -> Result<(), AIProxyError> {
        // tokenize string, return error if max token
        if let StoreInput::RawString(value) = input {
            //let tokenized_input;
            let model_embedding_dim = index_model.model_info().embedding_size;
            if value.len() > model_embedding_dim {
                if let StringAction::ErrorIfTokensExceed = string_action {
                    return Err(AIProxyError::TokenExceededError {
                        input_token_size: input.len(),
                        model_embedding_size: model_embedding_dim,
                    });
                } else {
                    // truncate raw string
                    // let tokenized_input;
                    value.as_str()[..model_embedding_dim].to_string();
                }
            }
        }
        Ok(())
    }

    fn process_image(
        &self,
        input: &mut StoreInput,
        index_model: &AIModel,
        image_action: &ImageAction,
    ) -> Result<(), AIProxyError> {
        // process image, return error if max dimensions exceeded
        if let StoreInput::Image(value) = input {
            // let image_data;
            let model_embedding_dim = index_model.embedding_size().into();
            if value.len() > model_embedding_dim {
                if let ImageAction::ErrorIfDimensionsMismatch = image_action {
                    return Err(AIProxyError::ImageDimensionsMismatchError {
                        image_dimensions: value.len(),
                        max_dimensions: model_embedding_dim,
                    });
                } else {
                    // resize image
                }
            }
        }
        Ok(())
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
