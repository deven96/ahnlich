use crate::error::AIProxyError;
use ahnlich_types::ai::AIModel;
use ahnlich_types::ai::AIStoreInfo;
use ahnlich_types::ai::AIStoreType;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::StoreName;
use ahnlich_types::metadata::MetadataKey;
use deadpool::managed::Pool;
use flurry::HashMap as ConcurrentHashMap;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;
use std::collections::HashSet as StdHashSet;
use std::mem::size_of_val;
use std::num::NonZeroUsize;
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
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIStore {
    name: StoreName,
    /// Making use of a concurrent hashmap, we should be able to create an engine that manages stores
    entries: ConcurrentHashMap<StoreName, StoreInput>,
    r#type: AIStoreType,
    model: AIModel,
}

impl AIStore {
    pub(super) fn create(r#type: AIStoreType, store_name: StoreName, model: AIModel) -> Self {
        Self {
            r#type,
            name: store_name,
            entries: ConcurrentHashMap::new(),
            model,
        }
    }

    pub(super) fn size(&self) -> usize {
        todo!()
    }
}
