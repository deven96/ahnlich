use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::processors::{
    Preprocessor, PreprocessorData, CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD,
};
use crate::error::AIProxyError;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub struct CenterCrop {
    crop_size: (u32, u32), // (width, height)
}

impl CenterCrop {
    pub fn initialize(config: &serde_json::Value) -> Result<Option<Self>, AIProxyError> {
        if !config["do_center_crop"].as_bool().unwrap_or(false) {
            return Ok(None);
        }

        let image_processor_type = config["image_processor_type"]
            .as_str()
            .unwrap_or("CLIPImageProcessor");

        match image_processor_type {
            "CLIPImageProcessor" => {
                let crop_size = &config["crop_size"];
                let has_crop_size = crop_size.is_object() || crop_size.is_u64();
                if !has_crop_size {
                    return Err(AIProxyError::ModelConfigLoadError {
                        message:
                        "The key 'crop_size' is missing from the configuration or has the wrong type; \
                        it should be an integer or an object containing 'height' and 'width' mappings.".to_string(),
                    });
                }
                let (width, height);
                if crop_size.is_object() {
                    height = crop_size["height"].as_u64().ok_or_else(|| {
                        AIProxyError::ModelConfigLoadError {
                            message:
                                "The key 'height' is missing from the ['crop_size'] section of \
                        the configuration or has the wrong type; it should be an integer"
                                    .to_string(),
                        }
                    })? as u32;
                    width = crop_size["width"].as_u64().ok_or_else(|| {
                        AIProxyError::ModelConfigLoadError {
                            message:
                                "The key 'width' is missing from the ['crop_size'] section of \
                        the configuration or has the wrong type; it should be an integer"
                                    .to_string(),
                        }
                    })? as u32;
                } else {
                    let size = crop_size
                        .as_u64()
                        .expect("It will always be an integer here.")
                        as u32;
                    width = size;
                    height = size;
                }

                Ok(Some(Self {
                    crop_size: (width, height),
                }))
            }
            "ConvNextFeatureExtractor" => {
                let size = &config["size"];
                if !size.is_object() {
                    return Err(AIProxyError::ModelConfigLoadError {
                        message: "The key 'size' is missing from the configuration or has the wrong type; it should be an object containing a 'shortest_edge' mapping.".to_string(),
                    });
                }
                let shortest_edge = size["shortest_edge"].as_u64().ok_or_else(|| {
                    AIProxyError::ModelConfigLoadError {
                        message: "The key 'shortest_edge' is missing from the ['size'] section of \
                        the configuration or has the wrong type; it should be an integer"
                            .to_string(),
                    }
                })? as u32;

                let should_center_crop =
                    shortest_edge < CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD;
                match should_center_crop {
                    true => Ok(Some(Self {
                        crop_size: (shortest_edge, shortest_edge),
                    })),
                    false => Ok(None),
                }
            }
            _ => Err(AIProxyError::ModelConfigLoadError {
                message: format!(
                    "The key 'image_processor_type' in the configuration has the wrong value: {}; \
                it should be either 'CLIPImageProcessor' or 'ConvNextFeatureExtractor'.",
                    image_processor_type
                )
                .to_string(),
            }),
        }
    }
}

impl Preprocessor for CenterCrop {
    #[tracing::instrument(skip_all)]
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(image_array) => {
                let processed = image_array
                    .into_par_iter()
                    .map(|mut image| {
                        let (width, height) = image.image_dim();
                        let width = width.get() as u32;
                        let height = height.get() as u32;
                        let (crop_width, crop_height) = self.crop_size;
                        if crop_width == width && crop_height == height {
                            Ok(image)
                        } else if crop_width <= width || crop_height <= height {
                            let x = (width - crop_width) / 2;
                            let y = (height - crop_height) / 2;
                            let image = image.crop(x, y, crop_width, crop_height)?;
                            Ok(image)
                        } else {
                            // The Fastembed-rs implementation pads the image with zeros, but that does not make
                            // sense to me (HAKSOAT), just as it does not make sense to "crop" to a bigger size.
                            // This is why I am going with resize, it is also important to note that
                            // I expect these cases to be minor because Resize will often be called before Center Crop anyway.
                            let image = image.resize(crop_width, crop_height, None)?;
                            Ok(image)
                        }
                    })
                    .collect::<Result<Vec<ImageArray>, AIProxyError>>();
                Ok(PreprocessorData::ImageArray(processed?))
            }
            _ => Err(AIProxyError::CenterCropError {
                message: "CenterCrop process failed. Expected ImageArray, got NdArray3C"
                    .to_string(),
            }),
        }
    }
}
