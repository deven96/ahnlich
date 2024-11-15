use image::imageops::FilterType;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::processors::{CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD, Processor, ProcessorData};
use crate::error::AIProxyError;

pub struct Resize {
    size: (u32, u32), // (width, height)
    resample: FilterType,
    process: bool
}

impl TryFrom<&serde_json::Value> for Resize {
    type Error = AIProxyError;

    fn try_from(config: &serde_json::Value) -> Result<Self, AIProxyError> {
        // let config = SafeValue::new(config.to_owned());
        if !config["do_resize"].as_bool().unwrap_or(false) {
            return Ok(
                Self {
                    size: (0, 0),
                    resample: FilterType::CatmullRom,
                    process: false
                }
            );
        }

        let image_processor_type = config["image_processor_type"].as_str()
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
        let has_value = shortest_edge.is_u64() ||
            (size_width.is_u64() && size_height.is_u64()
                && image_processor_type == "CLIPImageProcessor");
        if !has_value {
            return Err(AIProxyError::ModelConfigLoadError {
                message: "The ['size'] section of the configuration must contain either a \
                        'shortest_edge' mapping or 'width' and 'height' mappings (when \
                        'image_processor_type' is 'CLIPImageProcessor'); they should be \
                        integers.".to_string(),
            });
        }

        if shortest_edge.is_u64() {
            width = shortest_edge.as_u64().expect("It will always be an integer here.") as u32;
            height = width;
        } else {
            width = size_width.as_u64().expect("It will always be an integer here.") as u32;
            height = size_height.as_u64().expect("It will always be an integer here.") as u32;
        }

        match image_processor_type {
            "CLIPImageProcessor" => {
                Ok(Self { size: (width, height), resample: FilterType::CatmullRom, process: true })
            },
            "ConvNextFeatureExtractor" => {
                if width >= CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD {
                    Ok(Self { size: (width, height), resample: FilterType::CatmullRom, process: true
                    })
                } else {
                    let default_crop_pct = 0.875;
                    let crop_pct = config["crop_pct"].as_f64().unwrap_or(default_crop_pct) as f32;
                    let upsampled_edge = (width as f32 / crop_pct) as u32;
                    Ok(Self { size: (upsampled_edge, upsampled_edge), resample: FilterType::CatmullRom,
                        process: true })
                }
            },
            _ => Err(AIProxyError::ModelConfigLoadError {
                message: format!("Resize init failed. image_processor_type {} not supported", image_processor_type),
            })
        }
    }
}

impl Processor for Resize {
    fn process(&self, data: ProcessorData) -> Result<ProcessorData, AIProxyError> {
        match data {
            ProcessorData::ImageArray(mut arrays) => {
                let processed = arrays.par_iter_mut()
                    .map(|image| {
                        if !self.process {
                            return Ok(image.clone());
                        }

                        let image = image.resize(self.size.0, self.size.1, Some(self.resample))?;
                        Ok(image)
                    })
                    .collect::<Result<Vec<ImageArray>, AIProxyError>>();
                Ok(ProcessorData::ImageArray(processed?))
            }
            _ => Err(AIProxyError::ImageArrayToNdArrayError {
                message: "Resize failed. Expected ImageArray, got NdArray3C".to_string(),
            }),
        }
    }
}