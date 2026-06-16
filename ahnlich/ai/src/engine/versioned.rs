use std::collections::HashMap;
use std::sync::Arc;

use ahnlich_types::keyval::StoreName;
use ahnlich_types::schema::Schema;
use serde::{Deserialize, Serialize};
use utils::fallible;
use utils::persistence::{PersistenceTaskError, VersionedPersistence};

use super::store::{AIStore, AIStores};

pub const AI_CURRENT_VERSION: u32 = 2;
pub const AI_MIN_VERSION: u32 = 1;

type AiStoresV1 = HashMap<StoreName, AIStore>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "db_version")]
pub enum VersionedAiStores {
    #[serde(rename = "1")]
    V1 { stores: AiStoresV1 },

    #[serde(rename = "2")]
    V2 { stores: AIStores },
}

impl VersionedPersistence for VersionedAiStores {
    const CURRENT_VERSION: u32 = AI_CURRENT_VERSION;
    const MIN_VERSION: u32 = AI_MIN_VERSION;

    fn load_and_migrate(bytes: &[u8]) -> Result<Self, PersistenceTaskError> {
        Self::validate_version(bytes)?;

        match serde_json::from_slice::<VersionedAiStores>(bytes) {
            Ok(versioned) => Ok(versioned),
            Err(_) => {
                log::warn!("No db_version tag, attempting bare V1 load");
                let v1_stores: AiStoresV1 = serde_json::from_slice(bytes)?;
                Ok(VersionedAiStores::V1 { stores: v1_stores })
            }
        }
    }
}

impl VersionedAiStores {
    pub fn into_latest(self) -> Result<AIStores, String> {
        match self {
            VersionedAiStores::V1 { stores } => {
                log::info!("Migrating AI stores V1 → V2");
                Self::migrate_v1_to_v2(stores)
            }
            VersionedAiStores::V2 { stores } => Ok(stores),
        }
    }

    pub fn current(stores: AIStores) -> Self {
        VersionedAiStores::V2 { stores }
    }

    fn migrate_v1_to_v2(v1_stores: AiStoresV1) -> Result<AIStores, String> {
        let inner_stores =
            fallible::try_new_arc_hashmap().map_err(|e| format!("Migration failed: {e}"))?;
        {
            let guard = inner_stores.pin();
            for (name, store) in v1_stores {
                guard.insert(name, Arc::new(store));
            }
        }
        let stores =
            fallible::try_new_arc_hashmap().map_err(|e| format!("Migration failed: {e}"))?;
        {
            let guard = stores.pin();
            guard.insert(Schema::default(), inner_stores);
        }
        Ok(stores)
    }
}
