use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::processors::{
    CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD, Preprocessor, PreprocessorData,
};
use crate::error::AIProxyError;
use image::imageops::FilterType;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};

pub struct Resize {
    size: (u32, u32), // (width, height)
    resample: FilterType,
}

impl Resize {
    pub fn initialize(config: &serde_json::Value) -> Result<Option<Self>, AIProxyError> {
        if !config["do_resize"].as_bool().unwrap_or(false) {
            return Ok(None);
        }

        let image_processor_type = config["image_processor_type"]
            .as_str()
            .unwrap_or("CLIPImageProcessor");

        let size = &config["size"];
        if !size.is_object() {
            return Err(AIProxyError::ModelConfigLoadError {
                message: "The key 'size' is missing from the configuration or has the wrong type; \
                it should be an object containing a 'shortest_edge' mapping or a 'width' and 'height' mapping.".to_string(),
            });
        }

        let (width, height): (u32, u32);

        let shortest_edge = &size["shortest_edge"];
        let size_width = &size["width"];
        let size_height = &size["height"];
        let has_value = shortest_edge.is_u64()
            || (size_width.is_u64()
                && size_height.is_u64()
                && image_processor_type == "CLIPImageProcessor");
        if !has_value {
            return Err(AIProxyError::ModelConfigLoadError {
                message: "The ['size'] section of the configuration must contain either a \
                        'shortest_edge' mapping or 'width' and 'height' mappings (when \
                        'image_processor_type' is 'CLIPImageProcessor'); they should be \
                        integers."
                    .to_string(),
            });
        }

        if shortest_edge.is_u64() {
            width = shortest_edge
                .as_u64()
                .expect("It will always be an integer here.") as u32;
            height = width;
        } else {
            width = size_width
                .as_u64()
                .expect("It will always be an integer here.") as u32;
            height = size_height
                .as_u64()
                .expect("It will always be an integer here.") as u32;
        }

        match image_processor_type {
            "CLIPImageProcessor" => Ok(Some(Self {
                size: (width, height),
                resample: FilterType::CatmullRom,
            })),
            "ConvNextFeatureExtractor" => {
                if width >= CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD {
                    Ok(Some(Self {
                        size: (width, height),
                        resample: FilterType::CatmullRom,
                    }))
                } else {
                    let default_crop_pct = 0.875;
                    let crop_pct = config["crop_pct"].as_f64().unwrap_or(default_crop_pct) as f32;
                    let upsampled_edge = (width as f32 / crop_pct) as u32;
                    Ok(Some(Self {
                        size: (upsampled_edge, upsampled_edge),
                        resample: FilterType::CatmullRom,
                    }))
                }
            }
            _ => Err(AIProxyError::ModelConfigLoadError {
                message: format!(
                    "Resize init failed. image_processor_type {image_processor_type} not supported"
                ),
            }),
        }
    }
}

impl Preprocessor for Resize {
    #[tracing::instrument(skip_all)]
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(mut arrays) => {
                let processed = arrays
                    .par_iter_mut()
                    .map(|image| {
                        let resized_image =
                            image.resize(self.size.0, self.size.1, Some(self.resample))?;
                        Ok(resized_image)
                    })
                    .collect::<Result<Vec<ImageArray>, AIProxyError>>();
                Ok(PreprocessorData::ImageArray(processed?))
            }
            _ => Err(AIProxyError::ImageArrayToNdArrayError {
                message: "Resize failed. Expected ImageArray, got NdArray3C".to_string(),
            }),
        }
    }
}
