use crate::cli::server::SupportedModels;
use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::ort::helper::HFConfigReader;
use crate::engine::ai::providers::processors::center_crop::CenterCrop;
use crate::engine::ai::providers::processors::imagearray_to_ndarray::ImageArrayToNdArray;
use crate::engine::ai::providers::processors::normalize::ImageNormalize;
use crate::engine::ai::providers::processors::rescale::Rescale;
use crate::engine::ai::providers::processors::resize::Resize;
use crate::engine::ai::providers::processors::tokenize::{Tokenize, TokenizerFiles};
use crate::engine::ai::providers::processors::{AudioInput, Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use hf_hub::api::sync::ApiRepo;
use ndarray::{Array, Ix4};
use rubato::{FftFixedIn, Resampler};
use std::f32::consts::PI;
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

    /// Buffalo_L uses a hardcoded config because its weights come from insightface
    /// and don't ship a `preprocessor_config.json`. The values here match the
    /// RetinaFace detection head: 640×640 input, pixel range [-1, 1].
    fn load_buffalo_l(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        use serde_json::json;

        let imagearray_to_ndarray = ImageArrayToNdArray;

        let config = json!({
            "do_resize": true,
            "size": { "width": 640, "height": 640 },
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

/// Preprocessor for CLAP audio inputs. Mirrors the `rand_trunc` path of `ClapFeatureExtractor`
/// (which is what the Xenova ONNX export was built with):
///   raw bytes → decode → mono mixdown → resample to 48 kHz
///   → pad/repeat to 480 000 samples (10 s) → log-Mel spectrogram (64 bins, Slaney scale)
///   → truncate to 1000 frames → `AudioInput { input_features (B,1,1000,64) }`
///
/// Parameter values are read from `preprocessor_config.json` in the Xenova repo:
///   sampling_rate=48000, fft_window_size=1024, hop_length=480,
///   feature_size=64, frequency_min=50, frequency_max=14000, nb_max_frames=1000.
pub struct ORTAudioPreprocessor {
    model: SupportedModels,
    target_sample_rate: u32,
    /// 480 000 = 10 s × 48 000 Hz
    max_samples: usize,
    /// 1024-sample Hann window for STFT
    fft_window: usize,
    /// 480-sample hop (= 10 ms at 48 kHz)
    hop_length: usize,
    /// 64 mel filter bands
    n_mels: usize,
    /// Number of time frames the model expects: nb_max_samples / hop_length = 480000 / 480 = 1000
    nb_max_frames: usize,
    /// Slaney-normalised mel filterbank, shape (n_mels, n_freq_bins), precomputed at construction
    mel_filters_slaney: Vec<Vec<f32>>,
}

impl ORTAudioPreprocessor {
    pub fn new(model: SupportedModels, target_sample_rate: u32, clip_duration_secs: f32) -> Self {
        let fft_window = 1024_usize;
        let hop_length = 480_usize;
        let n_mels = 64_usize;
        let max_samples = (target_sample_rate as f32 * clip_duration_secs).round() as usize;
        // nb_max_frames = nb_max_samples / hop_length, as specified in preprocessor_config.json
        let nb_max_frames = max_samples / hop_length;
        let n_freq_bins = fft_window / 2 + 1;

        let mel_filters_slaney =
            build_mel_filterbank_slaney(n_freq_bins, n_mels, 50.0, 14_000.0, target_sample_rate);

        Self {
            model,
            target_sample_rate,
            max_samples,
            fft_window,
            hop_length,
            n_mels,
            nb_max_frames,
            mel_filters_slaney,
        }
    }

    /// Decodes raw audio bytes (any container format supported by symphonia) into a
    /// mono f32 waveform at the file's native sample rate. Multi-channel audio is
    /// mixed down by averaging all channels per frame.
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

        while let Ok(packet) = format.next_packet() {
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
            let mut padded = chunk.to_vec();
            padded.resize(chunk_size, 0.0);
            let resampled = resampler
                .process(&[padded], None)
                .map_err(|e| AIProxyError::AudioResampleError(e.to_string()))?;
            output.extend_from_slice(&resampled[0]);
        }
        Ok(output)
    }

    /// Pads short audio by repeating the waveform (then zero-padding the remainder),
    /// matching the `repeatpad` strategy from `ClapFeatureExtractor`.
    /// Returns the adjusted waveform and whether the original exceeded `max_samples`.
    fn pad_repeat(pcm: Vec<f32>, max_samples: usize) -> (Vec<f32>, bool) {
        let longer = pcm.len() > max_samples;
        if longer {
            let mut out = pcm;
            out.truncate(max_samples);
            (out, true)
        } else {
            let n = pcm.len();
            if n == max_samples {
                return (pcm, false);
            }
            let n_repeat = max_samples / n;
            let mut out = pcm.repeat(n_repeat);
            out.resize(max_samples, 0.0);
            (out, false)
        }
    }

    /// Computes a log-Mel spectrogram using a Hann-windowed STFT.
    ///
    /// Returns up to `nb_max_frames` frames, each of shape `(n_mels,)`.
    /// Energy values are converted to dB: `10 * log10(max(power, 1e-10))`.
    fn log_mel_spectrogram(&self, waveform: &[f32]) -> Vec<Vec<f32>> {
        let n_freq = self.fft_window / 2 + 1;
        let hann: Vec<f32> = hann_window(self.fft_window);

        let n_frames = if waveform.len() >= self.fft_window {
            ((waveform.len() - self.fft_window) / self.hop_length + 1).min(self.nb_max_frames)
        } else {
            0
        };

        let mut mel_frames: Vec<Vec<f32>> = Vec::with_capacity(n_frames);

        for frame_idx in 0..n_frames {
            let start = frame_idx * self.hop_length;
            let frame = &waveform[start..start + self.fft_window];

            let windowed: Vec<f32> = frame.iter().zip(hann.iter()).map(|(s, w)| s * w).collect();
            let power = real_dft_power(&windowed, n_freq);

            let mel_power: Vec<f32> = self
                .mel_filters_slaney
                .iter()
                .map(|filt| {
                    filt.iter()
                        .zip(power.iter())
                        .map(|(f, p)| f * p)
                        .sum::<f32>()
                })
                .collect();

            let mel_db: Vec<f32> = mel_power
                .iter()
                .map(|&p| 10.0 * p.max(1e-10_f32).log10())
                .collect();

            mel_frames.push(mel_db);
        }

        mel_frames
    }

    #[tracing::instrument(skip(self, data))]
    pub fn process(&self, data: Vec<Vec<u8>>) -> Result<AudioInput, AIProxyError> {
        let batch = data.len();
        let mut all_features: Vec<f32> =
            Vec::with_capacity(batch * self.nb_max_frames * self.n_mels);

        for bytes in &data {
            let (pcm, src_sr) = Self::decode_audio(bytes)?;
            let pcm = self.resample(pcm, src_sr)?;
            let (pcm, _longer) = Self::pad_repeat(pcm, self.max_samples);

            let mel = self.log_mel_spectrogram(&pcm);

            // Flatten frames into a single vector, zero-padding to nb_max_frames if short
            let mut flat = Vec::with_capacity(self.nb_max_frames * self.n_mels);
            for frame in &mel {
                flat.extend_from_slice(frame);
            }
            flat.resize(self.nb_max_frames * self.n_mels, 0.0);
            all_features.extend_from_slice(&flat);
        }

        // Shape: (batch, 1, nb_max_frames, n_mels) — the leading 1 is the view dimension
        // expected by the ONNX audio encoder's `input_features` input.
        let input_features =
            Array::from_shape_vec((batch, 1, self.nb_max_frames, self.n_mels), all_features)
                .map_err(|e| AIProxyError::ModelPreprocessingError {
                    model_name: self.model.to_string(),
                    message: format!("Failed to stack audio features: {e}"),
                })?;

        Ok(AudioInput {
            input_features,
            is_longer: vec![false; batch],
        })
    }
}

/// Standard Hann window: `0.5 * (1 - cos(2π·i/n))`.
fn hann_window(n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / n as f32).cos()))
        .collect()
}

/// Computes the one-sided power spectrum (magnitude squared) of a windowed frame
/// using the DFT definition directly. O(N²) — acceptable for the fixed N=1024 window.
/// Returns `n_freq = N/2 + 1` values.
fn real_dft_power(frame: &[f32], n_freq: usize) -> Vec<f32> {
    let n = frame.len();
    let mut power = vec![0.0_f32; n_freq];
    for (k, p) in power.iter_mut().enumerate() {
        let mut re = 0.0_f32;
        let mut im = 0.0_f32;
        let angle = -2.0 * PI * k as f32 / n as f32;
        for (t, &x) in frame.iter().enumerate() {
            re += x * (angle * t as f32).cos();
            im += x * (angle * t as f32).sin();
        }
        *p = re * re + im * im;
    }
    power
}

/// Builds a Slaney-normalised mel filterbank matrix of shape `(n_mels, n_freq_bins)`.
///
/// Uses the Slaney mel scale and area-normalises each triangular filter so that
/// all filters have equal energy contribution — matching `librosa.filters.mel` and
/// the `mel_filters_slaney` path in `ClapFeatureExtractor`, which is used when
/// `truncation = "rand_trunc"` (the mode used by the Xenova ONNX export).
fn build_mel_filterbank_slaney(
    n_freq_bins: usize,
    n_mels: usize,
    f_min: f32,
    f_max: f32,
    sample_rate: u32,
) -> Vec<Vec<f32>> {
    // Slaney mel scale: linear below 1000 Hz, logarithmic above
    let hz_to_mel = |f: f32| -> f32 { 2595.0 * (1.0 + f / 700.0).log10() };
    let mel_to_hz = |m: f32| -> f32 { 700.0 * (10.0_f32.powf(m / 2595.0) - 1.0) };

    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);

    // n_mels + 2 evenly-spaced mel points (the +2 are the left/right edges of the bank)
    let mel_points: Vec<f32> = (0..=n_mels + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
        .collect();

    let bin_points: Vec<usize> = mel_points
        .iter()
        .map(|&m| {
            let hz = mel_to_hz(m);
            let bin = (hz / sample_rate as f32 * 2.0 * (n_freq_bins - 1) as f32).round() as usize;
            bin.min(n_freq_bins - 1)
        })
        .collect();

    let mut filters = vec![vec![0.0_f32; n_freq_bins]; n_mels];
    for m in 0..n_mels {
        let left = bin_points[m];
        let center = bin_points[m + 1];
        let right = bin_points[m + 2];

        if center != left {
            for (k, v) in filters[m][left..center].iter_mut().enumerate() {
                *v = k as f32 / (center - left) as f32;
            }
        }
        if right != center {
            for (k, v) in filters[m][center..right].iter_mut().enumerate() {
                *v = (right - center - k) as f32 / (right - center) as f32;
            }
        }
        if center < n_freq_bins {
            filters[m][center] = 1.0;
        }

        // Slaney normalisation: divide by the width of the mel band in Hz
        let norm = mel_to_hz(mel_points[m + 2]) - mel_to_hz(mel_points[m]);
        if norm > 0.0 {
            for v in &mut filters[m] {
                *v *= 2.0 / norm;
            }
        }
    }

    filters
}
