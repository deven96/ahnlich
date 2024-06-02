use bincode::config::DefaultOptions;
use bincode::config::Options;
use serde::Deserialize;
use serde::Serialize;

pub const LENGTH_HEADER_SIZE: usize = 8;

/// - Length encoding must use fixed int and not var int
/// - Endianess must be Big Endian.
/// - First 8 bytes must contain length of the entire vec of response or queries
///
/// Used to serialize and deserialize queries and responses into bincode
pub trait BinCodeSerAndDeser<'a>
where
    Self: Serialize + Deserialize<'a>,
{
    fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian();
        let serialized_data = config.serialize(self)?;
        let data_length = serialized_data.len() as u64;
        // serialization appends the length buffer to be read first
        let mut buffer = Vec::with_capacity(LENGTH_HEADER_SIZE + serialized_data.len());
        buffer.extend(&data_length.to_be_bytes());
        buffer.extend(&serialized_data);
        Ok(buffer)
    }

    fn deserialize(has_length_header: bool, bytes: &'a [u8]) -> Result<Self, bincode::Error> {
        let config = DefaultOptions::new()
            .with_fixint_encoding()
            .with_big_endian();
        if has_length_header {
            return config.deserialize(&bytes[LENGTH_HEADER_SIZE..]);
        }
        config.deserialize(bytes)
    }
}
