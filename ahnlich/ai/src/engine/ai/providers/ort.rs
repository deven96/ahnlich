use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, Model, ModelInput};
use crate::engine::ai::providers::ort_helper::{get_tokenizer_artifacts_hf_hub, normalize,
                                               load_tokenizer_artifacts_hf_hub};
use crate::engine::ai::providers::{ProviderTrait, TextPreprocessorTrait};
use crate::error::AIProxyError;
use fallible_collections::FallibleVec;
use hf_hub::{api::sync::ApiBuilder, Cache};
use itertools::Itertools;
use rayon::iter::Either;
use ort::{Session, Value};
use tokenizers::Tokenizer;
use rayon::prelude::*;

use ahnlich_types::keyval::StoreKey;
use ndarray::{Array, Array1, ArrayView, Axis, Ix3};
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;

#[derive(Default)]
pub struct ORTProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    pub preprocessor: Option<ORTPreprocessor>,
    pub model: Option<ORTModel>,
}

impl fmt::Debug for ORTProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ORTProvider")
            .field("cache_location", &self.cache_location)
            .field("cache_location_extension", &self.cache_location_extension)
            .field("supported_models", &self.supported_models)
            .finish()
    }
}

#[derive(Default)]
pub struct ORTImageModel {
    repo_name: String,
    weights_file: String,
    session: Option<Session>,
    input_params: Vec<String>,
    output_param: String,
}

#[derive(Default)]
pub struct ORTTextModel {
    repo_name: String,
    weights_file: String,
    session: Option<Session>,
    input_params: Vec<String>,
    output_param: String,
}

pub enum ORTModel {
    Image(ORTImageModel),
    Text(ORTTextModel)
}

pub enum ORTPreprocessor {
    Text(ORTTextPreprocessor),
}

pub struct ORTTextPreprocessor {
    tokenizer: Tokenizer,
}

impl TryFrom<&SupportedModels> for ORTModel {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type: Result<ORTModel, AIProxyError> = match model {
            SupportedModels::Resnet50 => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                input_params: vec!["input".to_string()],
                output_param: "image_embeds".to_string(),
                ..Default::default()
            })),
            SupportedModels::ClipVitB32Image => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string(),
                input_params: vec!["pixel_values".to_string()],
                output_param: "image_embeds".to_string(),
                ..Default::default()
            })),
            SupportedModels::ClipVitB32Text => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Qdrant/clip-ViT-B-32-text".to_string(),
                weights_file: "model.onnx".to_string(),
                input_params: vec!["input_ids".to_string(), "attention_mask".to_string()],
                output_param: "text_embeds".to_string(),
                ..Default::default()
            })),
            _ => Err(AIProxyError::AIModelNotSupported {
                model_name: model.to_string(),
            }),
        };

        model_type
    }
}

impl ORTProvider {
    pub(crate) fn new() -> Self {
        Self {
            cache_location: None,
            cache_location_extension: PathBuf::from("huggingface"),
            preprocessor: None,
            supported_models: None,
            model: None,
        }
    }

    pub fn batch_inference_image(&self, mut inputs: Vec<ImageArray>) -> Result<Vec<StoreKey>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Image(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };

        let array_views: Vec<ArrayView<f32, Ix3>> = inputs
            .par_iter_mut()
            .map(|image_arr| {
                image_arr.onnx_transform();
                image_arr.view()
            })
            .collect();

        let pixel_values_array = ndarray::stack(ndarray::Axis(0), &array_views)
            .map_err(|e| AIProxyError::EmbeddingShapeError(e.to_string()))?;
        match &model.session {
            Some(session) => {
                let session_inputs = ort::inputs![
                    model.input_params.first().expect("Hardcoded in parameters")
                    .as_str() => pixel_values_array.view(),
                ].map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

                let outputs = session.run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let last_hidden_state_key = match outputs.len() {
                    1 => outputs.keys().next().unwrap(),
                    _ => model.output_param.as_str(),
                };

                let output_data = outputs[last_hidden_state_key]
                    .try_extract_tensor::<f32>()
                    .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
                let store_keys = output_data
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .map(|row| {
                        let embeddings = normalize(row.as_slice().unwrap());
                        StoreKey(<Array1<f32>>::from(embeddings))
                    })
                    .collect();
                Ok(store_keys)
            }
            None => Err(AIProxyError::AIModelNotInitialized)
        }
    }

    pub fn batch_inference_text(&self, inputs: Vec<String>) -> Result<Vec<StoreKey>, AIProxyError> {
        let inputs = inputs.iter().map(|x| x.as_str()).collect::<Vec<&str>>();
        let model = match &self.model {
            Some(ORTModel::Text(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };
        let batch_size = inputs.len();
        let encodings = match &self.preprocessor {
            Some(ORTPreprocessor::Text(preprocessor)) => {
                // TODO: We encode tokens at the preprocess step early in the workflow then also encode here.
                // Find a way to store those encoded tokens for reuse here.
                preprocessor.tokenizer.encode_batch(inputs, true).map_err(|_| {
                    AIProxyError::ModelTokenizationError
                })?
            }
            _ => return Err(AIProxyError::AIModelNotInitialized)
        };

        // Extract the encoding length and batch size
        let encoding_length = encodings[0].len();

        let max_size = encoding_length * batch_size;

        // Preallocate arrays with the maximum size
        let mut ids_array = Vec::with_capacity(max_size);
        let mut mask_array = Vec::with_capacity(max_size);
        let mut typeids_array = Vec::with_capacity(max_size);

        // Not using par_iter because the closure needs to be FnMut
        encodings.iter().for_each(|encoding| {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();
            let typeids = encoding.get_type_ids();

            // Extend the preallocated arrays with the current encoding
            // Requires the closure to be FnMut
            ids_array.extend(ids.iter().map(|x| *x as i64));
            mask_array.extend(mask.iter().map(|x| *x as i64));
            typeids_array.extend(typeids.iter().map(|x| *x as i64));
        });

        // Create CowArrays from vectors
        let inputs_ids_array =
            Array::from_shape_vec((batch_size, encoding_length), ids_array)
                .map_err(|e| {
                AIProxyError::ModelProviderPreprocessingError(e.to_string())
            })?;

        let attention_mask_array =
            Array::from_shape_vec((batch_size, encoding_length), mask_array).map_err(|e| {
                AIProxyError::ModelProviderPreprocessingError(e.to_string())
            })?;

        match &model.session {
            Some(session) => {
                let session_inputs = ort::inputs![
                    model.input_params.first()
                    .expect("Hardcoded in parameters").as_str() => Value::from_array(inputs_ids_array)?,
                    model.input_params.get(1)
                    .expect("Hardcoded in parameters").as_str() => Value::from_array(attention_mask_array.view())?
                ].map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;
                let outputs = session.run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let last_hidden_state_key = match outputs.len() {
                    1 => outputs
                        .keys()
                        .next()
                        .expect("Should not happen as length was checked"),
                    _ => model.output_param.as_str(),
                };

                let output_data = outputs[last_hidden_state_key]
                    .try_extract_tensor::<f32>()
                    .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
                let store_keys = output_data
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .map(|row| {
                        let embeddings = normalize(row.as_slice().unwrap());
                        StoreKey(<Array1<f32>>::from(embeddings))
                    })
                    .collect();
                Ok(store_keys)
            }
            None => Err(AIProxyError::AIModelNotInitialized),
        }
    }
}

impl TextPreprocessorTrait for ORTProvider {
    fn encode_str(&self, text: &str) -> Result<Vec<usize>, AIProxyError> {
        match &self.model {
            Some(ORTModel::Text(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };

        let Some(ORTPreprocessor::Text(preprocessor)) = &self.preprocessor else {
            return Err(AIProxyError::AIModelNotInitialized);
        };

        let tokens = preprocessor.tokenizer.encode(text, true)
            .map_err(|_| {AIProxyError::ModelTokenizationError})?;
        Ok(tokens.get_ids().iter().map(|x| *x as usize).collect())
    }

    fn decode_tokens(&self, tokens: Vec<usize>) -> Result<String, AIProxyError> {
        match &self.model {
            Some(ORTModel::Text(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };

        let Some(ORTPreprocessor::Text(preprocessor)) = &self.preprocessor else {
            return Err(AIProxyError::AIModelNotInitialized);
        };

        let tokens = tokens.iter().map(|x| *x as u32).collect::<Vec<u32>>();
        preprocessor.tokenizer.decode(&tokens, true)
            .map_err(|_| {AIProxyError::ModelTokenizationError})
    }
}

impl ProviderTrait for ORTProvider {
    fn set_cache_location(&mut self, location: &Path) {
        self.cache_location = Some(location.join(self.cache_location_extension.clone()));
    }

    fn set_model(&mut self, model: &SupportedModels) {
        self.supported_models = Some(*model);
    }

    fn load_model(&mut self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let Some(supported_model) = self.supported_models else {
            return Err(AIProxyError::AIModelNotInitialized);
        };
        let ort_model = ORTModel::try_from(&supported_model)?;

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        let threads = available_parallelism()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?
            .get();

        match ort_model {
            ORTModel::Image(ORTImageModel {
                weights_file,
                repo_name,
                                input_params: input_param,
                output_param,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                let session = Session::builder()?
                    .with_intra_threads(threads)?
                    .commit_from_file(model_file_reference)?;
                self.model = Some(ORTModel::Image(ORTImageModel {
                    repo_name,
                    weights_file,
                    input_params: input_param,
                    output_param,
                    session: Some(session),
                }));
            },
            ORTModel::Text(ORTTextModel {
                weights_file,
                repo_name,
                input_params,
                output_param,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                let session = Session::builder()?
                    .with_intra_threads(threads)?
                    .commit_from_file(model_file_reference)?;
                let max_token_length = Model::from(&(self.supported_models
                    .ok_or(AIProxyError::AIModelNotInitialized)?))
                    .max_input_token()
                    .ok_or(AIProxyError::AIModelNotInitialized)?;
                let tokenizer = load_tokenizer_artifacts_hf_hub(&model_repo,
                                                                usize::from(max_token_length))?;
                self.model = Some(ORTModel::Text(ORTTextModel {
                    repo_name,
                    weights_file,
                    input_params,
                    output_param,
                    session: Some(session),
                }));
                self.preprocessor = Some(ORTPreprocessor::Text(
                    ORTTextPreprocessor {
                    tokenizer,
                }));
            }
        }
        Ok(())
    }

    fn get_model(&self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let supported_model = self
            .supported_models
            .ok_or(AIProxyError::AIModelNotInitialized)?;
        let ort_model = ORTModel::try_from(&supported_model)?;

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        match ort_model {
            ORTModel::Image(ORTImageModel {
                repo_name,
                weights_file,
                ..
            }) => {
                let model_repo = api.model(repo_name);
                model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                model_repo
                    .get("preprocessor_config.json")
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                Ok(())
            },
            ORTModel::Text(ORTTextModel {
                repo_name,
                ..
            }) => {
                let model_repo = api.model(repo_name);
                get_tokenizer_artifacts_hf_hub(&model_repo)?;
                Ok(())
            }
        }
    }

    fn run_inference(
        &self,
        inputs: Vec<ModelInput>,
        _action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let (string_inputs, image_inputs): (Vec<String>, Vec<ImageArray>) =
            inputs.into_par_iter().partition_map(|input| match input {
                ModelInput::Text(value) => Either::Left(value),
                ModelInput::Image(value) => Either::Right(value),
            });

        if !image_inputs.is_empty() && !string_inputs.is_empty() {
            return Err(AIProxyError::VaryingInferenceInputTypes)
        }
        let batch_size = 16;
        let mut store_keys: Vec<_> = FallibleVec::try_with_capacity(
            image_inputs.len().max(string_inputs.len())
        )?;
        if !image_inputs.is_empty() {
            for batch_inputs in image_inputs.into_iter().chunks(batch_size).into_iter() {
                store_keys.extend(self.batch_inference_image(batch_inputs.collect())?);
            }
        } else {
            for batch_inputs in string_inputs.into_iter().chunks(batch_size).into_iter() {
                store_keys.extend(self.batch_inference_text(batch_inputs.collect())?);
            }
        }
        Ok(store_keys)
    }
}
