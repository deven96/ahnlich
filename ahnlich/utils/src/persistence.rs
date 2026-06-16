use memmap2::Mmap;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::BufWriter;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use task_manager::Task;
use task_manager::TaskState;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::time::Duration;
use tokio::time::sleep;

const MMAP_THRESHOLD: u64 = 64 * 1024; // 64KB

#[derive(Error, Debug)]
pub enum VersionError {
    #[error(
        "Persistence file version {file_version} is too new for this ahnlich binary (max supported: {max_version}). Please upgrade ahnlich."
    )]
    VersionTooNew { file_version: u32, max_version: u32 },

    #[error(
        "Persistence file version {file_version} is too old (minimum supported: {min_version}). Migration path removed."
    )]
    VersionTooOld { file_version: u32, min_version: u32 },
}

/// Trait for versioned persistence types.
pub trait VersionedPersistence: Sized + Serialize + for<'de> Deserialize<'de> {
    /// The current version this binary writes.
    const CURRENT_VERSION: u32;

    /// The minimum version this binary can read.
    const MIN_VERSION: u32;

    /// Validate version compatibility from raw bytes.
    fn validate_version(bytes: &[u8]) -> Result<(), VersionError> {
        #[derive(Deserialize)]
        struct VersionOnly {
            #[serde(rename = "db_version")]
            db_version: Option<String>,
        }

        let version = match serde_json::from_slice::<VersionOnly>(bytes) {
            Ok(v) => v
                .db_version
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1),
            Err(_) => 1,
        };

        if version > Self::CURRENT_VERSION {
            return Err(VersionError::VersionTooNew {
                file_version: version,
                max_version: Self::CURRENT_VERSION,
            });
        }

        if version < Self::MIN_VERSION {
            return Err(VersionError::VersionTooOld {
                file_version: version,
                min_version: Self::MIN_VERSION,
            });
        }

        Ok(())
    }

    /// Load from bytes with automatic migration from older formats.
    fn load_and_migrate(bytes: &[u8]) -> Result<Self, PersistenceTaskError>;
}

pub trait AhnlichPersistenceUtils {
    type PersistenceObject: Serialize + DeserializeOwned + Send + Sync + 'static + Debug;

    fn write_flag(&self) -> Arc<AtomicBool>;

    // TODO: We can in theory make loading of snapshot possible across threads but it is annoying
    // and not completely necessary(?) to have to lock and unlock a primitive to be able to modify
    // simply to load snapshot at the start

    //    fn use_snapshot(&self, object: Self::PersistenceObject);

    fn get_snapshot(&self) -> Self::PersistenceObject;
}

#[derive(Error, Debug)]
pub enum PersistenceTaskError {
    #[error("Error with file {0}")]
    FileError(#[from] std::io::Error),
    #[error("SerdeError {0}")]
    SerdeError(#[from] serde_json::error::Error),
    #[error("MigrationError {0}")]
    MigrationError(String),
    #[error("VersionError {0}")]
    Version(#[from] VersionError),
}

#[derive(Debug, Clone)]
pub struct Persistence<T> {
    write_flag: Arc<AtomicBool>,
    persistence_interval: u64,
    persist_object: T,
    persist_location: PathBuf,
}

#[async_trait::async_trait]
impl<T: Sync + Serialize + DeserializeOwned + Debug> Task for Persistence<T> {
    fn task_name(&self) -> String {
        "persistence".to_string()
    }

    async fn run(&self) -> TaskState {
        if self.has_potential_write().await {
            log::debug!("In potential write");
            let persist_location: &Path = self.persist_location.as_ref();
            let writer = match NamedTempFile::new_in(
                persist_location
                    .parent()
                    .expect("Could not get parent directory of persist location"),
            ) {
                Ok(file) => file,
                Err(e) => {
                    log::error!("Could not create persistence file with err {e:?}, skipping");
                    return TaskState::Continue;
                }
            };
            // set write flag to false before writing to it
            let _ =
                self.write_flag
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst);
            {
                let buf_writer = BufWriter::new(&writer);
                if let Err(e) = serde_json::to_writer(buf_writer, &self.persist_object) {
                    log::error!("Error writing stores to temp file {e:?}");
                    return TaskState::Continue;
                }
            }
            match writer.persist(persist_location) {
                Ok(_) => log::debug!("Persisted stores to disk"),
                Err(e) => log::error!("Error persisting temp file to location {e}"),
            }
        }
        TaskState::Continue
    }
}

impl<T: Serialize + DeserializeOwned> Persistence<T> {
    pub fn load_snapshot(
        persist_location: &std::path::PathBuf,
        enable_mmap: bool,
    ) -> Result<T, PersistenceTaskError> {
        let bytes = Self::read_snapshot_raw(persist_location, enable_mmap)?;
        Ok(serde_json::from_slice(&bytes)?)
    }

    pub fn read_snapshot_raw(
        persist_location: &std::path::PathBuf,
        enable_mmap: bool,
    ) -> Result<Vec<u8>, PersistenceTaskError> {
        let file = File::open(persist_location)?;
        let file_size = file.metadata()?.len();

        Ok(if enable_mmap && file_size > MMAP_THRESHOLD {
            log::debug!(
                "Using mmap to load persistence file (size: {} bytes)",
                file_size
            );
            let mmap = unsafe { Mmap::map(&file)? };
            mmap.to_vec()
        } else {
            log::debug!(
                "Using buffered reader to load persistence file (size: {} bytes)",
                file_size
            );
            let reader = BufReader::new(file);
            let mut bytes = Vec::with_capacity(file_size as usize);
            use std::io::Read;
            reader.take(file_size).read_to_end(&mut bytes)?;
            bytes
        })
    }

    pub fn load_snapshot_with_migration<F>(
        persist_location: &std::path::PathBuf,
        enable_mmap: bool,
        migrate: F,
    ) -> Result<T, PersistenceTaskError>
    where
        F: FnOnce(&[u8]) -> Result<T, PersistenceTaskError>,
    {
        match Self::load_snapshot(persist_location, enable_mmap) {
            Ok(snapshot) => Ok(snapshot),
            Err(_) => {
                let bytes = Self::read_snapshot_raw(persist_location, enable_mmap)?;
                migrate(&bytes)
            }
        }
    }

    pub fn task(
        write_flag: Arc<AtomicBool>,
        persistence_interval: u64,
        persist_location: &std::path::PathBuf,
        persist_object: T,
        _enable_mmap: bool,
    ) -> Self {
        let _ = OpenOptions::new()
            .append(true)
            .create(true)
            .open(persist_location)
            .expect("Persistence enabled but could not open peristence file");
        Self {
            write_flag,
            persistence_interval,
            persist_object,
            persist_location: persist_location.clone(),
        }
    }

    async fn has_potential_write(&self) -> bool {
        sleep(Duration::from_millis(self.persistence_interval)).await;
        self.write_flag.load(Ordering::SeqCst)
    }
}
