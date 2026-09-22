use std::collections::HashMap;
use std::sync::Arc;

use ahnlich_types::keyval::StoreName;
use ahnlich_types::schema::Schema;
use serde::{Deserialize, Serialize};
use utils::fallible;
#[cfg(feature = "server")]
use utils::persistence::{PersistenceTaskError, VersionedPersistence};

use super::store::{Store, Stores};

pub const DB_CURRENT_VERSION: u32 = 3;
pub const DB_MIN_VERSION: u32 = 1;

type DbStoresV1 = HashMap<StoreName, Store>;
#[cfg(feature = "server")]
type DbStoresV2 = HashMap<Schema, DbStoresV1>;

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
struct TaggedDbStoresV1 {
    stores: DbStoresV1,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
struct TaggedDbStoresV2 {
    stores: DbStoresV2,
}

#[cfg(feature = "server")]
#[derive(Debug, Deserialize)]
struct TaggedDbStoresV3 {
    stores: DbStoresV2,
}

#[cfg(feature = "server")]
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

    #[serde(rename = "3")]
    V3 { stores: Stores },
}

#[cfg(feature = "server")]
impl VersionedPersistence for VersionedDbStores {
    const CURRENT_VERSION: u32 = DB_CURRENT_VERSION;
    const MIN_VERSION: u32 = DB_MIN_VERSION;

    fn load_and_migrate(bytes: &[u8]) -> Result<Self, PersistenceTaskError> {
        Self::validate_version(bytes)?;

        match Self::snapshot_version(bytes) {
            Some(1) => {
                let tagged: TaggedDbStoresV1 = serde_json::from_slice(bytes)?;

                Ok(Self::V1 {
                    stores: tagged.stores,
                })
            }
            Some(2) => {
                let tagged: TaggedDbStoresV2 = serde_json::from_slice(bytes)?;

                let stores = Self::nested_to_stores(tagged.stores)
                    .map_err(PersistenceTaskError::MigrationError)?;

                Ok(Self::V2 { stores })
            }
            Some(3) => {
                let tagged: TaggedDbStoresV3 = serde_json::from_slice(bytes)?;

                let stores = Self::nested_to_stores(tagged.stores)
                    .map_err(PersistenceTaskError::MigrationError)?;

                Ok(Self::V3 { stores })
            }
            _ => {
                if let Ok(nested) = serde_json::from_slice::<DbStoresV2>(bytes) {
                    log::warn!("No db_version tag, detected schema-nested (V1n) format");

                    let stores = Self::nested_to_stores(nested)
                        .map_err(PersistenceTaskError::MigrationError)?;

                    return Ok(Self::V2 { stores });
                }

                log::warn!("No db_version tag, attempting bare V1 load");

                let stores: DbStoresV1 = serde_json::from_slice(bytes)?;

                Ok(Self::V1 { stores })
            }
        }
    }
}

#[cfg(feature = "server")]
impl VersionedDbStores {
    fn snapshot_version(bytes: &[u8]) -> Option<u32> {
        serde_json::from_slice::<DbVersionTag>(bytes)
            .ok()
            .and_then(|tag| tag.db_version)
            .and_then(|version| version.parse::<u32>().ok())
    }

    fn nested_to_stores(nested: DbStoresV2) -> Result<Stores, String> {
        let stores = fallible::try_new_arc_hashmap()
            .map_err(|error| format!("Migration failed: {error}"))?;

        for (schema, nested_stores) in nested {
            let inner_stores = fallible::try_new_arc_hashmap()
                .map_err(|error| format!("Migration failed: {error}"))?;

            {
                let guard = inner_stores.pin();

                for (name, store) in nested_stores {
                    guard.insert(name, Arc::new(store));
                }
            }

            {
                let guard = stores.pin();
                guard.insert(schema, inner_stores);
            }
        }

        Ok(stores)
    }
}

impl VersionedDbStores {
    pub fn into_latest(self) -> Result<Stores, String> {
        match self {
            Self::V1 { stores } => {
                log::info!("Migrating DB stores V1 → V2");

                let stores = Self::migrate_v1_to_v2(stores)?;

                log::info!("Migrating DB stores V2 → V3");

                Ok(Self::migrate_v2_to_v3(stores))
            }
            Self::V2 { stores } => {
                log::info!("Migrating DB stores V2 → V3");

                Ok(Self::migrate_v2_to_v3(stores))
            }
            Self::V3 { stores } => Ok(stores),
        }
    }

    pub fn current(stores: Stores) -> Self {
        Self::V3 { stores }
    }

    fn migrate_v1_to_v2(v1_stores: DbStoresV1) -> Result<Stores, String> {
        let inner_stores = fallible::try_new_arc_hashmap()
            .map_err(|error| format!("Migration failed: {error}"))?;

        {
            let guard = inner_stores.pin();

            for (name, store) in v1_stores {
                guard.insert(name, Arc::new(store));
            }
        }

        let stores = fallible::try_new_arc_hashmap()
            .map_err(|error| format!("Migration failed: {error}"))?;

        {
            let guard = stores.pin();
            guard.insert(Schema::default(), inner_stores);
        }

        Ok(stores)
    }

    fn migrate_v2_to_v3(stores: Stores) -> Stores {
        {
            let schemas = stores.guard();

            for (_, inner_stores) in stores.iter(&schemas) {
                let inner = inner_stores.guard();

                for (_, store) in inner_stores.iter(&inner) {
                    store.rebuild_ordered_key_index();
                }
            }
        }

        stores
    }
}
