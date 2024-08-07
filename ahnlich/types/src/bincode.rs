use crate::version::VERSION;
use bincode::config::DefaultOptions;
use bincode::config::Options;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub const MAGIC_BYTES: &[u8] = b"AHNLICH;";
/// corresponds to the size of the Version struct
/// which is 8-16-16 bits or 5 bytes
pub const VERSION_LENGTH: usize = 5;
pub const LENGTH_HEADER_SIZE: usize = 8;
/// corresponds to the entire header length
pub const RESPONSE_HEADER_LEN: usize = MAGIC_BYTES.len() + VERSION_LENGTH + LENGTH_HEADER_SIZE;

/// - Length encoding must use fixed int and not var int
/// - Endianess must be Little Endian.
/// - First 8 bytes must be the magic bytes
/// - Followed by 5 bytes marking the version used. Server should typically agree to service
/// request where major version matches according to semver
/// - Next 8 bytes must contain length N of the entire vec of response or queries
/// - Final N bytes contain the vec of response or queries
///
/// Used to serialize and deserialize queries and responses into bincode
pub trait BinCodeSerAndDeser
where
    Self: Serialize + DeserializeOwned + Send,
{
    fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_little_endian();
        let serialized_version_data = config.serialize(&*VERSION)?;
        let serialized_data = config.serialize(self)?;
        let data_length = serialized_data.len() as u64;
        // serialization appends the length buffer to be read first
        let mut buffer = Vec::with_capacity(
            MAGIC_BYTES.len() + VERSION_LENGTH + LENGTH_HEADER_SIZE + serialized_data.len(),
        );
        buffer.extend(MAGIC_BYTES);
        buffer.extend(serialized_version_data);
        buffer.extend(&data_length.to_le_bytes());
        buffer.extend(&serialized_data);
        Ok(buffer)
    }

    fn deserialize(bytes: &[u8]) -> Result<Self, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_little_endian();
        config.deserialize(bytes)
    }
}

pub trait BinCodeSerAndDeserQuery: BinCodeSerAndDeser
where
    Self::Inner: Serialize + DeserializeOwned,
{
    type Inner;
    fn into_inner(self) -> Self::Inner;
}

pub trait BinCodeSerAndDeserResponse: BinCodeSerAndDeser {
    fn from_error(err: String) -> Self;
}
