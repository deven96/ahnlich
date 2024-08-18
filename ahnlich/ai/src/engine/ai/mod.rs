pub mod models;
use once_cell::sync::Lazy;

use ahnlich_types::ai::AIModel;

pub(crate) static AHNLICH_AI_SUPPORTED_MODELS: Lazy<Vec<AIModel>> =
    Lazy::new(|| vec![AIModel::Llama3, AIModel::DALLE3]);
