use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::ort::ORTProvider;
use crate::engine::ai::providers::ModelProviders;
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::ai::ExecutionProvider;
use ahnlich_types::{ai::AIStoreInputType, keyval::StoreKey};
use fast_image_resize::images::Image;
use fast_image_resize::images::ImageRef;
use fast_image_resize::FilterType;
use fast_image_resize::PixelType;
use fast_image_resize::ResizeAlg;
use fast_image_resize::ResizeOptions;
use fast_image_resize::Resizer;
use image::imageops;
use image::ImageReader;
use image::RgbImage;
use ndarray::{Array, Ix3};
use ndarray::{ArrayView, Ix4};
use nonzero_ext::nonzero;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Cursor;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use strum::Display;
use tokenizers::Encoding;

static CHANNELS: Lazy<u8> = Lazy::new(|| image::ColorType::Rgb8.channel_count());

#[derive(Display, Debug, Serialize, Deserialize)]
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
    pub provider: ModelProviders,
    pub model_details: ModelDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelDetails {
    pub model_type: ModelType,
    pub description: String,
    pub supported_model: SupportedModels,
    pub embedding_size: NonZeroUsize,
}

impl SupportedModels {
    pub fn to_model_details(&self) -> ModelDetails {
        match self {
            SupportedModels::AllMiniLML6V2 => ModelDetails {
                model_type: ModelType::Text {
                    max_input_tokens: nonzero!(256usize),
                },
                supported_model: SupportedModels::AllMiniLML6V2,
                description: String::from("Sentence Transformer model, with 6 layers, version 2"),
                embedding_size: nonzero!(384usize),
            },
            SupportedModels::AllMiniLML12V2 => ModelDetails {
                    model_type: ModelType::Text {
                        // Token size source: https://huggingface.co/sentence-transformers/all-MiniLM-L12-v2#intended-uses
                        max_input_tokens: nonzero!(256usize),
                    },
                    supported_model: SupportedModels::AllMiniLML12V2,
                    description: String::from("Sentence Transformer model, with 12 layers, version 2."),
                    embedding_size: nonzero!(384usize),
            },
            SupportedModels::BGEBaseEnV15 => ModelDetails {
                    model_type: ModelType::Text {
                        // Token size source: https://huggingface.co/BAAI/bge-large-en/discussions/11#64e44de1623074ac850aa1ae
                        max_input_tokens: nonzero!(512usize),
                    },
                    supported_model: SupportedModels::BGEBaseEnV15,
                    description: String::from(
                        "BAAI General Embedding model with English support, base scale, version 1.5.",
                    ),
                    embedding_size: nonzero!(768usize),
            },
            SupportedModels::BGELargeEnV15 => ModelDetails {
                    model_type: ModelType::Text {
                        max_input_tokens: nonzero!(512usize),
                    },
                    supported_model: SupportedModels::BGELargeEnV15,
                    description: String::from(
                        "BAAI General Embedding model with English support, large scale, version 1.5.",
                    ),
                    embedding_size: nonzero!(1024usize),
            },
            SupportedModels::Resnet50 => ModelDetails {
                    model_type: ModelType::Image {
                        expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                    },
                    supported_model: SupportedModels::Resnet50,
                    description: String::from("Residual Networks model, with 50 layers."),
                    embedding_size: nonzero!(2048usize),
            },
            SupportedModels::ClipVitB32Image => ModelDetails {
                    model_type: ModelType::Image {
                        expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                    },
                    supported_model: SupportedModels::ClipVitB32Image,
                    description: String::from(
                        "Contrastive Language-Image Pre-Training Vision transformer model, base scale.",
                    ),
                    embedding_size: nonzero!(512usize),
            },
            SupportedModels::ClipVitB32Text => ModelDetails {
                    supported_model: SupportedModels::ClipVitB32Text,
                    description: String::from(
                        "Contrastive Language-Image Pre-Training Text transformer model, base scale. \
                            Ideal for embedding very short text and using in combination with ClipVitB32Image",
                    ),
                    embedding_size: nonzero!(512usize),
                    model_type: ModelType::Text {
                        // Token size source: https://github.com/UKPLab/sentence-transformers/issues/1269
                        max_input_tokens: nonzero!(77usize),
                    },
                },
        }
    }

    pub async fn to_concrete_model(&self, cache_location: PathBuf) -> Result<Model, AIProxyError> {
        let model_details = self.to_model_details();
        // can only be created with a cache location, this ties together the model public
        // facing details as well as the provider
        // if there are multiple providers, feel free to match here and override
        let provider = ModelProviders::ORT(
            ORTProvider::from_model_and_cache_location(self, cache_location).await?,
        );
        Ok(Model {
            model_details,
            provider,
        })
    }
}

impl ModelDetails {
    #[tracing::instrument(skip(self))]
    pub fn input_type(&self) -> AIStoreInputType {
        match self.model_type {
            ModelType::Text { .. } => AIStoreInputType::RawString,
            ModelType::Image { .. } => AIStoreInputType::Image,
        }
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
}

impl Model {
    #[tracing::instrument(skip(self))]
    pub fn input_type(&self) -> AIStoreInputType {
        self.model_details.input_type()
    }

    #[tracing::instrument(skip(self, modelinput))]
    pub async fn model_ndarray(
        &self,
        modelinput: ModelInput,
        action_type: &InputAction,
        execution_provider: Option<ExecutionProvider>,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let store_keys = match &self.provider {
            ModelProviders::ORT(provider) => {
                provider
                    .run_inference(modelinput, action_type, execution_provider)
                    .await?
            }
        };
        Ok(store_keys)
    }

    #[tracing::instrument(skip(self))]
    pub fn max_input_token(&self) -> Option<NonZeroUsize> {
        self.model_details.max_input_token()
    }

    #[tracing::instrument(skip(self))]
    pub fn expected_image_dimensions(&self) -> Option<(NonZeroUsize, NonZeroUsize)> {
        self.model_details.expected_image_dimensions()
    }

    #[tracing::instrument(skip(self))]
    pub fn model_name(&self) -> String {
        self.model_details.model_name()
    }

    pub async fn get(&self) -> Result<(), AIProxyError> {
        match &self.provider {
            ModelProviders::ORT(provider) => {
                provider.get_model().await?;
            }
        }
        Ok(())
    }
}

impl From<&Model> for AIStoreInputType {
    fn from(value: &Model) -> Self {
        match value.model_details.model_type {
            ModelType::Text { .. } => AIStoreInputType::RawString,
            ModelType::Image { .. } => AIStoreInputType::Image,
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

#[derive(Debug)]
pub struct OnnxTransformResult {
    array: Array<f32, Ix3>,
}

impl OnnxTransformResult {
    pub fn view(&self) -> ArrayView<f32, Ix3> {
        self.array.view()
    }

    pub fn image_dim(&self) -> (NonZeroUsize, NonZeroUsize) {
        let shape = self.array.shape();
        (
            NonZeroUsize::new(shape[2]).expect("Array columns should be non zero"),
            NonZeroUsize::new(shape[1]).expect("Array channels should be non zero"),
        )
    }
}

impl TryFrom<ImageArray> for OnnxTransformResult {
    type Error = AIProxyError;

    // Swapping axes from [rows, columns, channels] to [channels, rows, columns] for ONNX
    #[tracing::instrument(skip_all)]
    fn try_from(value: ImageArray) -> Result<Self, Self::Error> {
        let image = value.image;
        let mut array = Array::from_shape_vec(
            (
                image.height() as usize,
                image.width() as usize,
                *CHANNELS as usize,
            ),
            image.into_raw(),
        )
        .map_err(|e| AIProxyError::ImageArrayToNdArrayError {
            message: format!("Error running onnx transform {e}"),
        })?
        .mapv(f32::from);
        array.swap_axes(1, 2);
        array.swap_axes(0, 1);
        Ok(Self { array })
    }
}

#[derive(Debug)]
pub struct ImageArray {
    image: RgbImage,
}

impl TryFrom<&[u8]> for ImageArray {
    type Error = AIProxyError;

    #[tracing::instrument(skip_all)]
    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let img_reader = ImageReader::new(Cursor::new(value))
            .with_guessed_format()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;

        // Always convert to RGB8 format
        // https://github.com/Anush008/fastembed-rs/blob/cea92b6c8b877efda762393848d1c449a4eea126/src/image_embedding/utils.rs#L198
        let image = img_reader
            .decode()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?
            .into_rgb8();

        let (width, height) = image.dimensions();

        if width == 0 || height == 0 {
            return Err(AIProxyError::ImageNonzeroDimensionError {
                width: width as usize,
                height: height as usize,
            });
        }
        Ok(Self { image })
    }
}

impl ImageArray {
    fn array_view(&self) -> ArrayView<u8, Ix3> {
        let shape = (
            self.image.height() as usize,
            self.image.width() as usize,
            *CHANNELS as usize,
        );
        let raw_bytes = self.image.as_raw();
        ArrayView::from_shape(shape, raw_bytes).expect("Image bytes decode error")
    }

    #[tracing::instrument(skip(self))]
    pub fn resize(
        &mut self,
        width: u32,
        height: u32,
        filter: Option<image::imageops::FilterType>,
    ) -> Result<Self, AIProxyError> {
        // Create container for data of destination image
        let (width, height) = self.image.dimensions();
        let mut dest_image = Image::new(width, height, PixelType::U8x3);
        let mut resizer = Resizer::new();
        resizer
            .resize(
                &ImageRef::new(width, height, self.image.as_raw(), PixelType::U8x3)
                    .map_err(|e| AIProxyError::ImageResizeError(e.to_string()))?,
                &mut dest_image,
                &ResizeOptions::new().resize_alg(ResizeAlg::Convolution(FilterType::CatmullRom)),
            )
            .map_err(|e| AIProxyError::ImageResizeError(e.to_string()))?;
        let resized_img = RgbImage::from_raw(width, height, dest_image.into_vec())
            .expect("Could not get image after resizing");
        Ok(ImageArray { image: resized_img })
    }

    #[tracing::instrument(skip(self))]
    pub fn crop(&mut self, x: u32, y: u32, width: u32, height: u32) -> Result<Self, AIProxyError> {
        let cropped_img = imageops::crop(&mut self.image, x, y, width, height).to_image();

        Ok(ImageArray { image: cropped_img })
    }

    pub fn image_dim(&self) -> (NonZeroUsize, NonZeroUsize) {
        let arr_view = self.array_view();
        let shape = arr_view.shape();
        (
            NonZeroUsize::new(shape[1]).expect("Array columns should be non-zero"),
            NonZeroUsize::new(shape[0]).expect("Array rows should be non-zero"),
        )
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
