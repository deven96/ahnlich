use super::AudioInput;

/// Metadata for a single audio chunk produced during preprocessing.
///
/// Used to track temporal information when long audio is split into
/// 10-second segments with 1-second overlap.
#[derive(Debug)]
pub struct ChunkMetadata {
    /// Preprocessed audio input tensor (log-mel spectrogram)
    pub input: AudioInput,
    /// Start time in seconds within the original audio
    pub start_sec: f32,
    /// End time in seconds within the original audio
    pub end_sec: f32,
    /// Duration of actual audio (before padding) in seconds
    pub duration_sec: f32,
    /// Zero-based index of this chunk
    pub chunk_index: usize,
    /// Total number of chunks for this audio clip
    pub total_chunks: usize,
    /// Total duration of the original audio in seconds
    pub audio_total_duration_sec: f32,
}
