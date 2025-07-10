use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt::Debug;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use task_manager::Task;
use task_manager::TaskState;
use tempfile::NamedTempFile;
use thiserror::Error;
use tokio::time::Duration;
use tokio::time::sleep;

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
}

#[derive(Debug, Clone)]
pub struct Persistence<T> {
    write_flag: Arc<AtomicBool>,
    persistence_interval: u64,
    persist_location: std::path::PathBuf,
    persist_object: T,
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
            let writer = if let Ok(file) = NamedTempFile::new_in(
                persist_location
                    .parent()
                    .expect("Could not get parent directory of persist location"),
            ) {
                file
            } else {
                log::error!("Could not create persistence file, skipping");
                return TaskState::Continue;
            };
            let temp_path = writer.path();
            // set write flag to false before writing to it
            let _ =
                self.write_flag
                    .compare_exchange(true, false, Ordering::SeqCst, Ordering::SeqCst);
            if let Err(e) = serde_json::to_writer(&writer, &self.persist_object) {
                log::error!("Error writing stores to temp file {e:?}");
            } else {
                match std::fs::rename(temp_path, persist_location) {
                    Ok(_) => log::debug!("Persisted stores to disk"),
                    Err(e) => log::error!("Error writing temp file to persist location {e}"),
                };
            }
        }
        TaskState::Continue
    }
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
