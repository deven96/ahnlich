use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::ort::ORTProvider;
use crate::engine::ai::providers::ModelProviders;
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::StoreKey,
};
use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader};
use ndarray::{Array, Ix3};
use ndarray::{ArrayView, Ix4};
use nonzero_ext::nonzero;
use serde::de::Error as DeError;
use serde::ser::Error as SerError;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::path::Path;
use strum::Display;
use tokenizers::Encoding;

#[derive(Display)]
pub enum ModelType {
    Text {
        max_input_tokens: NonZeroUsize,
    },
    Image {
        // width, height
        expected_image_dimensions: (NonZeroUsize, NonZeroUsize),
    },
}

pub struct Model {
    pub model_type: ModelType,
    pub provider: ModelProviders,
    pub description: String,
    pub supported_model: SupportedModels,
    pub embedding_size: NonZeroUsize,
}

impl From<&AIModel> for Model {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::AllMiniLML6V2 => Self {
                model_type: ModelType::Text {
                    max_input_tokens: nonzero!(256usize),
                },
                provider: ModelProviders::ORT(ORTProvider::new(128)),
                supported_model: SupportedModels::AllMiniLML6V2,
                description: String::from("Sentence Transformer model, with 6 layers, version 2"),
                embedding_size: nonzero!(384usize),
            },
            AIModel::AllMiniLML12V2 => Self {
                model_type: ModelType::Text {
                    // Token size source: https://huggingface.co/sentence-transformers/all-MiniLM-L12-v2#intended-uses
                    max_input_tokens: nonzero!(256usize),
                },
                provider: ModelProviders::ORT(ORTProvider::new(128)),
                supported_model: SupportedModels::AllMiniLML12V2,
                description: String::from("Sentence Transformer model, with 12 layers, version 2."),
                embedding_size: nonzero!(384usize),
            },
            AIModel::BGEBaseEnV15 => Self {
                model_type: ModelType::Text {
                    // Token size source: https://huggingface.co/BAAI/bge-large-en/discussions/11#64e44de1623074ac850aa1ae
                    max_input_tokens: nonzero!(512usize),
                },
                provider: ModelProviders::ORT(ORTProvider::new(128)),
                supported_model: SupportedModels::BGEBaseEnV15,
                description: String::from(
                    "BAAI General Embedding model with English support, base scale, version 1.5.",
                ),
                embedding_size: nonzero!(768usize),
            },
            AIModel::BGELargeEnV15 => Self {
                model_type: ModelType::Text {
                    max_input_tokens: nonzero!(512usize),
                },
                provider: ModelProviders::ORT(ORTProvider::new(128)),
                supported_model: SupportedModels::BGELargeEnV15,
                description: String::from(
                    "BAAI General Embedding model with English support, large scale, version 1.5.",
                ),
                embedding_size: nonzero!(1024usize),
            },
            AIModel::Resnet50 => Self {
                model_type: ModelType::Image {
                    expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                },
                provider: ModelProviders::ORT(ORTProvider::new(16)),
                supported_model: SupportedModels::Resnet50,
                description: String::from("Residual Networks model, with 50 layers."),
                embedding_size: nonzero!(2048usize),
            },
            AIModel::ClipVitB32Image => Self {
                model_type: ModelType::Image {
                    expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                },
                provider: ModelProviders::ORT(ORTProvider::new(16)),
                supported_model: SupportedModels::ClipVitB32Image,
                description: String::from(
                    "Contrastive Language-Image Pre-Training Vision transformer model, base scale.",
                ),
                embedding_size: nonzero!(512usize),
            },
            AIModel::ClipVitB32Text => Self {
                model_type: ModelType::Text {
                    // Token size source: https://github.com/UKPLab/sentence-transformers/issues/1269
                    max_input_tokens: nonzero!(77usize),
                },
                provider: ModelProviders::ORT(ORTProvider::new(16)),
                supported_model: SupportedModels::ClipVitB32Text,
                description: String::from(
                    "Contrastive Language-Image Pre-Training Text transformer model, base scale. \
                    Ideal for embedding very short text and using in combination with ClipVitB32Image",
                ),
                embedding_size: nonzero!(512usize),
            },
        }
    }
}

impl Model {
    #[tracing::instrument(skip(self))]
    pub fn input_type(&self) -> AIStoreInputType {
        match self.model_type {
            ModelType::Text { .. } => AIStoreInputType::RawString,
            ModelType::Image { .. } => AIStoreInputType::Image,
        }
    }

    #[tracing::instrument(skip(self, modelinput))]
    pub fn model_ndarray(
        &self,
        modelinput: ModelInput,
        action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let store_keys = match &self.provider {
            ModelProviders::ORT(provider) => provider.run_inference(modelinput, action_type)?,
        };
        Ok(store_keys)
    }

    #[tracing::instrument(skip(self))]
    pub fn max_input_token(&self) -> Option<NonZeroUsize> {
        match self.model_type {
            ModelType::Text {
                max_input_tokens, ..
            } => Some(max_input_tokens),
            ModelType::Image { .. } => None,
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn expected_image_dimensions(&self) -> Option<(NonZeroUsize, NonZeroUsize)> {
        match self.model_type {
            ModelType::Text { .. } => None,
            ModelType::Image {
                expected_image_dimensions: (width, height),
                ..
            } => Some((width, height)),
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn model_name(&self) -> String {
        self.supported_model.to_string()
    }

    pub fn setup_provider(&mut self, cache_location: &Path) {
        let supported_model = self.supported_model;
        match &mut self.provider {
            ModelProviders::ORT(provider) => {
                provider.set_model(&supported_model);
                provider.set_cache_location(cache_location);
            }
        }
    }

    pub fn load(&mut self) -> Result<(), AIProxyError> {
        match &mut self.provider {
            ModelProviders::ORT(provider) => {
                provider.load_model()?;
            }
        }
        Ok(())
    }

    pub fn get(&self) -> Result<(), AIProxyError> {
        match &self.provider {
            ModelProviders::ORT(provider) => {
                provider.get_model()?;
            }
        }
        Ok(())
    }
}

impl From<&Model> for AIStoreInputType {
    fn from(value: &Model) -> Self {
        match value.model_type {
            ModelType::Text { .. } => AIStoreInputType::RawString,
            ModelType::Image { .. } => AIStoreInputType::Image,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModelInfo {
    name: String,
    input_type: AIStoreInputType,
    embedding_size: NonZeroUsize,
    max_input_tokens: Option<NonZeroUsize>,
    expected_image_dimensions: Option<(NonZeroUsize, NonZeroUsize)>,
    description: String,
}

impl ModelInfo {
    pub(crate) fn build(model: &Model) -> Self {
        Self {
            name: model.model_name(),
            input_type: model.input_type(),
            embedding_size: model.embedding_size,
            max_input_tokens: model.max_input_token(),
            expected_image_dimensions: model.expected_image_dimensions(),
            description: model.description.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputAction {
    Query,
    Index,
}

impl fmt::Display for InputAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Index => write!(f, "index"),
        }
    }
}

#[derive(Debug)]
pub enum ModelInput {
    Texts(Vec<Encoding>),
    Images(Array<f32, Ix4>),
}

#[derive(Debug, Clone)]
pub struct ImageArray {
    array: Array<f32, Ix3>,
    image: DynamicImage,
    image_format: ImageFormat,
    onnx_transformed: bool,
}

impl ImageArray {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, AIProxyError> {
        let img_reader = ImageReader::new(Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;

        let image_format = &img_reader
            .format()
            .ok_or(AIProxyError::ImageBytesDecodeError)?;

        let image = img_reader
            .decode()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;

        // Always convert to RGB8 format
        // https://github.com/Anush008/fastembed-rs/blob/cea92b6c8b877efda762393848d1c449a4eea126/src/image_embedding/utils.rs#L198
        let image: DynamicImage = image.to_owned().into_rgb8().into();
        let (width, height) = image.dimensions();

        if width == 0 || height == 0 {
            return Err(AIProxyError::ImageNonzeroDimensionError {
                width: width as usize,
                height: height as usize,
            });
        }

        let channels = &image.color().channel_count();
        let shape = (height as usize, width as usize, *channels as usize);
        let array = Array::from_shape_vec(shape, image.clone().into_bytes())
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?
            .mapv(f32::from);

        Ok(ImageArray {
            array,
            image,
            image_format: image_format.to_owned(),
            onnx_transformed: false,
        })
    }

    // Swapping axes from [rows, columns, channels] to [channels, rows, columns] for ONNX
    pub fn onnx_transform(&mut self) {
        if self.onnx_transformed {
            return;
        }
        self.array.swap_axes(1, 2);
        self.array.swap_axes(0, 1);
        self.onnx_transformed = true;
    }

    pub fn view(&self) -> ArrayView<f32, Ix3> {
        self.array.view()
    }

    pub fn get_bytes(&self) -> Result<Vec<u8>, AIProxyError> {
        let mut buffer = Cursor::new(Vec::new());
        let _ = &self
            .image
            .write_to(&mut buffer, self.image_format)
            .map_err(|_| AIProxyError::ImageBytesEncodeError)?;
        let bytes = buffer.into_inner();
        Ok(bytes)
    }

    pub fn resize(
        &self,
        width: u32,
        height: u32,
        filter: Option<image::imageops::FilterType>,
    ) -> Result<Self, AIProxyError> {
        let filter_type = filter.unwrap_or(image::imageops::FilterType::CatmullRom);
        let resized_img = self.image.resize_exact(width, height, filter_type);
        let channels = resized_img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);

        let flattened_pixels = resized_img.clone().into_bytes();
        let array = Array::from_shape_vec(shape, flattened_pixels)
            .map_err(|_| AIProxyError::ImageResizeError)?
            .mapv(f32::from);
        Ok(ImageArray {
            array,
            image: resized_img,
            image_format: self.image_format,
            onnx_transformed: false,
        })
    }

    pub fn crop(&self, x: u32, y: u32, width: u32, height: u32) -> Result<Self, AIProxyError> {
        let cropped_img = self.image.crop_imm(x, y, width, height);
        let channels = cropped_img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);

        let flattened_pixels = cropped_img.clone().into_bytes();
        let array = Array::from_shape_vec(shape, flattened_pixels)
            .map_err(|_| AIProxyError::ImageCropError)?
            .mapv(f32::from);
        Ok(ImageArray {
            array,
            image: cropped_img,
            image_format: self.image_format,
            onnx_transformed: false,
        })
    }

    pub fn image_dim(&self) -> (NonZeroUsize, NonZeroUsize) {
        let shape = self.array.shape();
        match self.onnx_transformed {
            true => (
                NonZeroUsize::new(shape[2]).expect("Array columns should be non-zero"),
                NonZeroUsize::new(shape[1]).expect("Array channels should be non-zero"),
            ), // (width, channels)
            false => (
                NonZeroUsize::new(shape[1]).expect("Array columns should be non-zero"),
                NonZeroUsize::new(shape[0]).expect("Array rows should be non-zero"),
            ), // (width, height)
        }
    }
}

impl Serialize for ImageArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.get_bytes().map_err(S::Error::custom)?)
    }
}

impl<'de> Deserialize<'de> for ImageArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        ImageArray::try_new(bytes).map_err(D::Error::custom)
    }
}

impl From<&ModelInput> for AIStoreInputType {
    fn from(value: &ModelInput) -> AIStoreInputType {
        match value {
            ModelInput::Texts(_) => AIStoreInputType::RawString,
            ModelInput::Images(_) => AIStoreInputType::Image,
        }
    }
}
