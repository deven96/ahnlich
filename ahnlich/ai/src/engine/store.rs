use crate::error::AIProxyError;
use crate::AHNLICH_AI_RESERVED_META_KEY;
use ahnlich_types::ai::AIModel;
use ahnlich_types::ai::AIStoreInfo;
use ahnlich_types::ai::AIStoreType;
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

    #[tracing::instrument(skip(self))]
    pub(crate) fn create_store(
        &self,
        store_name: StoreName,
        store_type: AIStoreType,
        model: AIModel,
    ) -> Result<(), AIProxyError> {
        self.stores
            .try_insert(
                store_name.clone(),
                Arc::new(AIStore::create(
                    store_type,
                    store_name.clone(),
                    model.clone(),
                )),
                &self.stores.guard(),
            )
            .map_err(|_| AIProxyError::StoreAlreadyExists(store_name.clone()))?;

        Ok(())
    }

    /// matches LISTSTORES - to return statistics of all stores
    #[tracing::instrument(skip(self))]
    pub(crate) fn list_stores(&self) -> StdHashSet<AIStoreInfo> {
        self.stores
            .iter(&self.stores.guard())
            .map(|(store_name, store)| AIStoreInfo {
                name: store_name.clone(),
                model: store.model.clone(),
                r#type: store.r#type.clone(),
                embedding_size: store.model.embedding_size().into(),
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
    /// Fails if the type of storeinput does not match the store type
    #[tracing::instrument(skip(self))]
    pub(crate) fn store_input_to_store_key_val(
        &self,
        store_name: &StoreName,
        store_input: StoreInput,
        store_value: &StoreValue,
    ) -> Result<(StoreKey, StoreValue), AIProxyError> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        if store_value.contains_key(metadata_key) {
            return Err(AIProxyError::ReservedError(metadata_key.to_string()));
        }
        let store = self.get(store_name)?;
        let input_type = store_input.clone().into();

        if store.r#type != input_type {
            return Err(AIProxyError::StoreTypeMismatch {
                store_type: store.r#type.clone(),
                input_type,
            });
        }
        let store_key = store.model.model_ndarray(&store_input);
        let metadata_value: MetadataValue = store_input.into();
        let mut final_store_value: StdHashMap<MetadataKey, MetadataValue> =
            store_value.clone().into_iter().collect();
        final_store_value.insert(metadata_key.clone(), metadata_value);
        return Ok((store_key, final_store_value));
    }

    /// Converts (storekey, storevalue) into (storeinput, storevalue)
    /// by removing the reserved_key from storevalue
    #[tracing::instrument(skip(self))]
    pub(crate) fn store_key_val_to_store_input_val(
        &self,
        output: Vec<(StoreKey, StoreValue)>,
    ) -> Vec<(StoreInput, StoreValue)> {
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;

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
        Ok(store.model.model_ndarray(store_input))
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
            //self.set_write_flag();
            1
        };
        Ok(removed)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIStore {
    name: StoreName,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    r#type: AIStoreType,
    model: AIModel,
}

impl AIStore {
    pub(super) fn create(r#type: AIStoreType, store_name: StoreName, model: AIModel) -> Self {
        Self {
            r#type,
            name: store_name,
            model,
        }
    }
}
