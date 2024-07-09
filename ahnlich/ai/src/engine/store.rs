use crate::error::AIProxyError;
use crate::AHNLICH_RESERVED_AI_META;
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
                size_in_bytes: store.size(),
            })
            .collect()
    }

    /// Returns a store using the store name, else returns an error
    #[tracing::instrument(skip(self))]
    fn get(&self, store_name: &StoreName) -> Result<Arc<AIStore>, AIProxyError> {
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
        let metadata_key = MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string());
        if store_value.contains_key(&metadata_key) {
            return Err(AIProxyError::ReservedError(metadata_key.to_string()));
        }
        let store = self.get(store_name)?;
        let input_type = store_input.clone().into();
        if store.r#type.clone() != input_type {
            return Err(AIProxyError::StoreTypeMismatch {
                store_type: store.r#type.clone(),
                input_type,
            });
        }
        let store_key = store.model.model_ndarray(&store_input);
        let metadata_value: MetadataValue = store_input.into();
        return Ok((
            store_key,
            StdHashMap::from_iter([(metadata_key, metadata_value)]),
        ));
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

    pub(super) fn size(&self) -> usize {
        todo!()
    }
}
