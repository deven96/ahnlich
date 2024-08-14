mod models;
use ahnlich_types::keyval::{StoreInput, StoreKey};
use models::ModelInfo;
use std::num::NonZeroUsize;

pub trait AIModelManager {
    fn embedding_size(&self) -> NonZeroUsize;
    fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey;
    fn model_info(&self) -> ModelInfo;
}
