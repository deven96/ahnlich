use std::collections::HashMap;
use std::sync::Arc;

use ahnlich_types::keyval::StoreName;
use ahnlich_types::schema::Schema;
use serde::{Deserialize, Serialize};
use utils::fallible;
use utils::persistence::{PersistenceTaskError, VersionedPersistence};

use super::store::{Store, Stores};

pub const DB_CURRENT_VERSION: u32 = 2;
pub const DB_MIN_VERSION: u32 = 1;

type DbStoresV1 = HashMap<StoreName, Store>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "db_version")]
pub enum VersionedDbStores {
    #[serde(rename = "1")]
    V1 { stores: DbStoresV1 },

    #[serde(rename = "2")]
    V2 { stores: Stores },
}

impl VersionedPersistence for VersionedDbStores {
    const CURRENT_VERSION: u32 = DB_CURRENT_VERSION;
    const MIN_VERSION: u32 = DB_MIN_VERSION;

    fn load_and_migrate(bytes: &[u8]) -> Result<Self, PersistenceTaskError> {
        Self::validate_version(bytes)?;

        match serde_json::from_slice::<VersionedDbStores>(bytes) {
            Ok(versioned) => Ok(versioned),
            Err(_) => {
                log::warn!("No db_version tag, attempting bare V1 load");
                let v1_stores: DbStoresV1 = serde_json::from_slice(bytes)?;
                Ok(VersionedDbStores::V1 { stores: v1_stores })
            }
        }
    }
}

impl VersionedDbStores {
    pub fn into_latest(self) -> Result<Stores, String> {
        match self {
            VersionedDbStores::V1 { stores } => {
                log::info!("Migrating DB stores V1 → V2");
                Self::migrate_v1_to_v2(stores)
            }
            VersionedDbStores::V2 { stores } => Ok(stores),
        }
    }

    pub fn current(stores: Stores) -> Self {
        VersionedDbStores::V2 { stores }
    }

    fn migrate_v1_to_v2(v1_stores: DbStoresV1) -> Result<Stores, String> {
        let inner_stores = fallible::try_new_arc_hashmap()
            .map_err(|e| format!("Migration failed: {e}"))?;
        {
            let guard = inner_stores.pin();
            for (name, store) in v1_stores {
                guard.insert(name, Arc::new(store));
            }
        }
        let stores = fallible::try_new_arc_hashmap()
            .map_err(|e| format!("Migration failed: {e}"))?;
        {
            let guard = stores.pin();
            guard.insert(Schema::default(), inner_stores);
        }
        Ok(stores)
    }
}
