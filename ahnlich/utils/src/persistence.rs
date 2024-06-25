use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::select;
use tokio::time::sleep;
use tokio::time::Duration;
use tokio_graceful::ShutdownGuard;

#[derive(Error, Debug)]
pub enum PersistenceTaskError {
    #[error("Error with file {0}")]
    FileError(#[from] std::io::Error),
    #[error("SerdeError {0}")]
    SerdeError(#[from] serde_json::error::Error),
}

pub struct Persistence<T> {
    write_flag: Arc<AtomicBool>,
    persistence_interval: u64,
    persist_location: std::path::PathBuf,
    persist_object: T,
}

impl<T: Serialize + DeserializeOwned> Persistence<T> {
    pub fn load_snapshot(persist_location: &std::path::PathBuf) -> Result<T, PersistenceTaskError> {
        let file = File::open(persist_location)?;
        let reader = BufReader::new(file);
        let loaded: T = serde_json::from_reader(reader)?;
        Ok(loaded)
    }

    pub fn task(
        write_flag: Arc<AtomicBool>,
        persistence_interval: u64,
        persist_location: &std::path::PathBuf,
        persist_object: T,
    ) -> Self {
        let _ = File::create(persist_location)
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

    #[tracing::instrument(skip(self))]
    pub async fn monitor(&mut self, shutdown_guard: ShutdownGuard) {
        loop {
            select! {
                _  = shutdown_guard.cancelled() => {
                    break;
                }
                has_potential_write = self.has_potential_write() => {
                    if has_potential_write {
                        let persist_location: &Path = self.persist_location.as_ref();
                        let writer = if let Ok(file) = NamedTempFile::new_in(persist_location.parent().expect("Could not get parent directory of persist location")) {
                            file
                        } else {
                            tracing::error!("Could not create persistence file, skipping");
                            continue;
                        };
                        let temp_path = writer.path();
                        // set write flag to false before writing to it
                        let _ = self.write_flag.compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst);
                        if let Err(e) = serde_json::to_writer(&writer, &self.persist_object) {
                            tracing::error!("Error writing stores to temp file {e}");

                        } else {
                            match std::fs::rename(temp_path, persist_location) {
                                Ok(_) => tracing::debug!("Persisted stores to disk"),
                                Err(e) => tracing::error!("Error writing temp file to persist location {e}"),
                            };
                        }
                    } else {
                        tracing::debug!("No potential writes happened during persistence interval")
                    }
                }
            }
        }
    }
}
