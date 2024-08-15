use ahnlich_types::keyval::{StoreInput, StoreKey};
pub(crate) mod models;
use models::ModelInfo;
use std::num::NonZeroUsize;

pub trait AIModelManager {
    fn embedding_size(&self) -> NonZeroUsize;
    fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey;
    fn model_info(&self) -> ModelInfo;
}
