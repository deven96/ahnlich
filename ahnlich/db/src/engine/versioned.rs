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
type DbStoresV2 = HashMap<Schema, DbStoresV1>;

#[derive(Debug, Deserialize)]
struct TaggedDbStoresV1 {
    stores: DbStoresV1,
}

#[derive(Debug, Deserialize)]
struct TaggedDbStoresV2 {
    stores: DbStoresV2,
}

#[derive(Debug, Deserialize)]
struct DbVersionTag {
    db_version: Option<String>,
}

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

        match Self::snapshot_version(bytes) {
            Some(1) => {
                let tagged: TaggedDbStoresV1 = serde_json::from_slice(bytes)?;
                Ok(VersionedDbStores::V1 {
                    stores: tagged.stores,
                })
            }
            Some(2) => {
                let tagged: TaggedDbStoresV2 = serde_json::from_slice(bytes)?;
                let stores = Self::migrate_v1n_to_v2(tagged.stores)
                    .map_err(PersistenceTaskError::MigrationError)?;
                Ok(VersionedDbStores::V2 { stores })
            }
            _ => {
                if let Ok(nested) = serde_json::from_slice::<DbStoresV2>(bytes) {
                    log::warn!("No db_version tag, detected schema-nested (V1n) format");
                    let stores = Self::migrate_v1n_to_v2(nested)
                        .map_err(PersistenceTaskError::MigrationError)?;
                    return Ok(VersionedDbStores::V2 { stores });
                }
                log::warn!("No db_version tag, attempting bare V1 load");
                let v1_stores: DbStoresV1 = serde_json::from_slice(bytes)?;
                Ok(VersionedDbStores::V1 { stores: v1_stores })
            }
        }
    }
}

impl VersionedDbStores {
    fn snapshot_version(bytes: &[u8]) -> Option<u32> {
        serde_json::from_slice::<DbVersionTag>(bytes)
            .ok()
            .and_then(|tag| tag.db_version)
            .and_then(|version| version.parse::<u32>().ok())
    }

    fn migrate_v1n_to_v2(v1_nested: DbStoresV2) -> Result<Stores, String> {
        let stores =
            fallible::try_new_arc_hashmap().map_err(|e| format!("Migration failed: {e}"))?;
        for (schema_name, inner) in v1_nested {
            let inner_stores =
                fallible::try_new_arc_hashmap().map_err(|e| format!("Migration failed: {e}"))?;
            {
                let guard = inner_stores.pin();
                for (name, store) in inner {
                    guard.insert(name, Arc::new(store));
                }
            }
            {
                let guard = stores.pin();
                guard.insert(schema_name, inner_stores);
            }
        }
        Ok(stores)
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
