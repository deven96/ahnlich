use crate::cli::server::SupportedModels;
use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::ort::helper::HFConfigReader;
use crate::engine::ai::providers::processors::center_crop::CenterCrop;
use crate::engine::ai::providers::processors::imagearray_to_ndarray::ImageArrayToNdArray;
use crate::engine::ai::providers::processors::normalize::ImageNormalize;
use crate::engine::ai::providers::processors::rescale::Rescale;
use crate::engine::ai::providers::processors::resize::Resize;
use crate::engine::ai::providers::processors::tokenize::{Tokenize, TokenizerFiles};
use crate::engine::ai::providers::processors::{AudioWaveform, Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use hf_hub::api::sync::ApiRepo;
use ndarray::{Array, Ix4};
use rubato::{FftFixedIn, Resampler};
use std::sync::{Arc, Mutex};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use tokenizers::Encoding;

pub enum ORTPreprocessor {
    Image(ORTImagePreprocessor),
    Text(ORTTextPreprocessor),
    Audio(ORTAudioPreprocessor),
}

pub struct ORTImagePreprocessor {
    model: SupportedModels,
    imagearray_to_ndarray: ImageArrayToNdArray,
    normalize: Option<ImageNormalize>,
    resize: Option<Resize>,
    rescale: Option<Rescale>,
    center_crop: Option<CenterCrop>,
}

impl ORTImagePreprocessor {
    pub fn load(
        supported_model: SupportedModels,
        model_repo: ApiRepo,
    ) -> Result<Self, AIProxyError> {
        let imagearray_to_ndarray = ImageArrayToNdArray;

        // Special handling for Buffalo_L - use hardcoded config
        if supported_model == SupportedModels::BuffaloL {
            return Self::load_buffalo_l(supported_model);
        }

        let mut config_reader = HFConfigReader::new(model_repo);
        let config = config_reader.read("preprocessor_config.json")?;

        let resize = Resize::initialize(&config)?;
        let center_crop = CenterCrop::initialize(&config)?;
        let rescale = Rescale::initialize(&config)?;
        let normalize = ImageNormalize::initialize(&config)?;

        Ok(Self {
            model: supported_model,
            imagearray_to_ndarray,
            normalize,
            resize,
            rescale,
            center_crop,
        })
    }

    /// Buffalo_L-specific preprocessor - resize to 640x640 for RetinaFace detection
    fn load_buffalo_l(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        use serde_json::json;

        let imagearray_to_ndarray = ImageArrayToNdArray;

        // Create config for RetinaFace detection (640x640 input)
        let config = json!({
            "do_resize": true,
            "size": {
                "width": 640,
                "height": 640
            },
            "image_processor_type": "CLIPImageProcessor",
            "do_normalize": true,
            "image_mean": [127.5, 127.5, 127.5],
            "image_std": [128.0, 128.0, 128.0]
        });

        let resize = Resize::initialize(&config)?;
        let normalize = ImageNormalize::initialize(&config)?;

        Ok(Self {
            model: supported_model,
            imagearray_to_ndarray,
            normalize,
            resize,
            rescale: None,
            center_crop: None,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn process(&self, data: Vec<ImageArray>) -> Result<Array<f32, Ix4>, AIProxyError> {
        let mut data = PreprocessorData::ImageArray(data);

        data = match self.resize {
            Some(ref resize) => {
                resize
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process resize: {e}"),
                    })?
            }
            None => data,
        };

        data =
            match self.center_crop {
                Some(ref center_crop) => center_crop.process(data).map_err(|e| {
                    AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process center crop: {e}"),
                    }
                })?,
                None => data,
            };

        data = self.imagearray_to_ndarray.process(data).map_err(|e| {
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: format!("Failed to process imagearray to ndarray: {e}"),
            }
        })?;

        data = match self.rescale {
            Some(ref rescale) => {
                rescale
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process rescale: {e}"),
                    })?
            }
            None => data,
        };

        data = match self.normalize {
            Some(ref normalize) => {
                normalize
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process normalize: {e}"),
                    })?
            }
            None => data,
        };

        match data {
            PreprocessorData::NdArray3C(array) => Ok(array),
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: "Expected NdArray3C after processing".to_string(),
            }),
        }
    }
}

pub struct ORTTextPreprocessor {
    model: SupportedModels,
    tokenize: Arc<Mutex<Tokenize>>,
}

impl ORTTextPreprocessor {
    pub fn load(
        supported_models: SupportedModels,
        model_repo: ApiRepo,
    ) -> Result<ORTTextPreprocessor, AIProxyError> {
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: "tokenizer.json".to_string(),
            config_file: "config.json".to_string(),
            special_tokens_map_file: "special_tokens_map.json".to_string(),
            tokenizer_config_file: "tokenizer_config.json".to_string(),
        };

        Ok(ORTTextPreprocessor {
            model: supported_models,
            tokenize: Arc::new(Mutex::new(Tokenize::initialize(
                tokenizer_files,
                model_repo,
            )?)),
        })
    }

    #[tracing::instrument(skip(self, data))]
    pub fn process(
        &self,
        data: Vec<String>,
        truncate: bool,
    ) -> Result<Vec<Encoding>, AIProxyError> {
        let mut data = PreprocessorData::Text(data);
        let mut tokenize =
            self.tokenize
                .lock()
                .map_err(|_| AIProxyError::ModelPreprocessingError {
                    model_name: self.model.to_string(),
                    message: "Failed to acquire lock on tokenize.".to_string(),
                })?;
        let _ = tokenize.set_truncate(truncate);
        data = tokenize
            .process(data)
            .map_err(|e| AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: format!("Failed to process tokenize: {e}"),
            })?;

        match data {
            PreprocessorData::EncodedText(encodings) => Ok(encodings),
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: "Expected EncodedText after processing".to_string(),
            }),
        }
    }
}

/// CLAP audio preprocessor: decode → mono → resample to target_sr → pad/truncate → stack
pub struct ORTAudioPreprocessor {
    model: SupportedModels,
    /// Target sample rate in Hz (48000 for CLAP)
    target_sample_rate: u32,
    /// Fixed clip duration in seconds (10s for CLAP, giving 480_000 samples at 48kHz)
    clip_duration_secs: f32,
}

impl ORTAudioPreprocessor {
    pub fn new(model: SupportedModels, target_sample_rate: u32, clip_duration_secs: f32) -> Self {
        Self {
            model,
            target_sample_rate,
            clip_duration_secs,
        }
    }

    /// Decode raw audio bytes (any symphonia-supported format) into a mono f32 waveform
    /// at the file's native sample rate.
    fn decode_audio(bytes: &[u8]) -> Result<(Vec<f32>, u32), AIProxyError> {
        let cursor = std::io::Cursor::new(bytes.to_vec());
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());
        let hint = Hint::new();
        let format_opts = FormatOptions::default();
        let metadata_opts = MetadataOptions::default();
        let decoder_opts = DecoderOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &format_opts, &metadata_opts)
            .map_err(|e| AIProxyError::AudioBytesDecodeError(e.to_string()))?;

        let mut format = probed.format;
        let track = format
            .default_track()
            .ok_or_else(|| AIProxyError::AudioBytesDecodeError("No audio track found".into()))?;

        let sample_rate = track
            .codec_params
            .sample_rate
            .ok_or_else(|| AIProxyError::AudioBytesDecodeError("Unknown sample rate".into()))?;
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
        let track_id = track.id;

        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &decoder_opts)
            .map_err(|e| AIProxyError::AudioBytesDecodeError(e.to_string()))?;

        let mut pcm: Vec<f32> = Vec::new();

        loop {
            let packet = match format.next_packet() {
                Ok(p) => p,
                Err(_) => break,
            };
            if packet.track_id() != track_id {
                continue;
            }
            let decoded = match decoder.decode(&packet) {
                Ok(d) => d,
                Err(_) => continue,
            };
            let spec = *decoded.spec();
            let mut sample_buf = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
            sample_buf.copy_interleaved_ref(decoded);
            let samples = sample_buf.samples();

            // Mix down to mono by averaging channels
            if channels == 1 {
                pcm.extend_from_slice(samples);
            } else {
                for frame in samples.chunks(channels) {
                    let mono = frame.iter().sum::<f32>() / channels as f32;
                    pcm.push(mono);
                }
            }
        }

        Ok((pcm, sample_rate))
    }

    /// Resample `pcm` from `src_sr` to `self.target_sample_rate` using an FFT-based resampler
    fn resample(&self, pcm: Vec<f32>, src_sr: u32) -> Result<Vec<f32>, AIProxyError> {
        if src_sr == self.target_sample_rate {
            return Ok(pcm);
        }
        let chunk_size = 1024;
        let mut resampler = FftFixedIn::<f32>::new(
            src_sr as usize,
            self.target_sample_rate as usize,
            chunk_size,
            2,
            1,
        )
        .map_err(|e| AIProxyError::AudioResampleError(e.to_string()))?;

        let mut output = Vec::new();
        for chunk in pcm.chunks(chunk_size) {
            // Pad the last chunk if needed
            let mut padded = chunk.to_vec();
            padded.resize(chunk_size, 0.0);
            let resampled = resampler
                .process(&[padded], None)
                .map_err(|e| AIProxyError::AudioResampleError(e.to_string()))?;
            output.extend_from_slice(&resampled[0]);
        }
        Ok(output)
    }

    /// Pad with zeros or truncate to exactly `target_len` samples
    fn pad_or_truncate(mut pcm: Vec<f32>, target_len: usize) -> Vec<f32> {
        pcm.resize(target_len, 0.0);
        pcm
    }

    #[tracing::instrument(skip(self, data))]
    pub fn process(&self, data: Vec<Vec<u8>>) -> Result<AudioWaveform, AIProxyError> {
        let target_len =
            (self.target_sample_rate as f32 * self.clip_duration_secs).round() as usize;

        let waveforms: Result<Vec<Vec<f32>>, AIProxyError> = data
            .iter()
            .map(|bytes| {
                let (pcm, src_sr) = Self::decode_audio(bytes)?;
                let resampled = self.resample(pcm, src_sr)?;
                Ok(Self::pad_or_truncate(resampled, target_len))
            })
            .collect();

        let waveforms = waveforms?;
        let batch = waveforms.len();

        // Stack into (batch, samples) ndarray
        let flat: Vec<f32> = waveforms.into_iter().flatten().collect();
        Array::from_shape_vec((batch, target_len), flat).map_err(|e| {
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: format!("Failed to stack audio waveforms: {e}"),
            }
        })
    }
}
