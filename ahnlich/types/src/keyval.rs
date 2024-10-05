use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use crate::errors::TypeError;
use ndarray::{Array1, Array, Ix3};
use image::{ImageReader, GenericImageView};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::collections::HashMap as StdHashMap;
use std::fmt;
use std::io::Cursor;
use std::num::NonZeroUsize;


/// Name of a Store
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreName(pub String);

impl fmt::Display for StoreName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A store value for now is a simple key value pair of strings
pub type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A store key is always an f32 one dimensional array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreKey(pub Array1<f32>);

impl StoreKey {
    pub fn dimension(&self) -> usize {
        self.0.len()
    }
}

impl Eq for StoreKey {}

impl PartialEq for StoreKey {
    fn eq(&self, other: &Self) -> bool {
        if self.0.shape() != other.0.shape() {
            return false;
        }
        // std::f32::EPSILON adheres to the IEEE 754 standard and we use it here to determine when
        // two Array1<f32> are extremely similar to the point where the differences are neglible.
        // We can modify to allow for greater precision, however we currently only
        // use it for PartialEq and not for it's distinctive properties. For that, within the
        // server we defer to using StoreKeyId whenever we want to compare distinctive Array1<f32>
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(x, y)| (x - y).abs() < f32::EPSILON)
    }
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ImageArray {
    array: Array<u8, Ix3>,
    bytes: Vec<u8>
}

impl ImageArray {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, TypeError> {
        let img_reader = ImageReader::new(Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|_| TypeError::ImageBytesDecodeError)?;

        let img = img_reader
            .decode()
            .map_err(|_| TypeError::ImageBytesDecodeError)?;
        let (width, height) = img.dimensions();
        let channels = img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);
        let array = Array::from_shape_vec(shape, img.into_bytes())
            .map_err(|_| TypeError::ImageBytesDecodeError)?;
        Ok(ImageArray { array, bytes })
    }

    pub fn get_array(&self) -> &Array<u8, Ix3> {
        &self.array
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn resize(&self, width: usize, height: usize) -> Result<Self, TypeError> {
        let img_reader = ImageReader::new(Cursor::new(&self.bytes))
            .with_guessed_format()
            .map_err(|_| TypeError::ImageBytesDecodeError)?;
        let img_format = img_reader.format().ok_or(TypeError::ImageBytesDecodeError)?;
        let original_img = img_reader
            .decode()
            .map_err(|_| TypeError::ImageBytesDecodeError)?;

        let resized_img = original_img.resize_exact(width as u32, height as u32,
                                                    image::imageops::FilterType::Triangle);
        let channels = resized_img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);

        let mut buffer = Cursor::new(Vec::new());
        resized_img.write_to(&mut buffer, img_format)
            .map_err(|_| TypeError::ImageResizeError)?;

        let flattened_pixels = resized_img.into_bytes();
        let array = Array::from_shape_vec(shape, flattened_pixels)
            .map_err(|_| TypeError::ImageResizeError)?;
        let bytes = buffer.into_inner();
        Ok(ImageArray { array, bytes })
    }

    pub fn image_dim(&self) -> InputLength {
        let shape = self.array.shape();
        InputLength::Image {
            width: NonZeroUsize::new(shape[1]).unwrap(),
            height: NonZeroUsize::new(shape[0]).unwrap(),
        }
    }
}


impl Serialize for ImageArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.get_bytes())
    }
}

impl<'de> Deserialize<'de> for ImageArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Ok(ImageArray::try_new(bytes).map_err(serde::de::Error::custom)?)
    }
}

impl Ord for ImageArray {
    fn cmp(&self, other: &Self) -> Ordering {
        let (array_vec, _) = self.array.clone().into_raw_vec_and_offset();
        let (other_vec, _) = other.array.clone().into_raw_vec_and_offset();
        array_vec.cmp(&other_vec)
    }
}

impl PartialOrd for ImageArray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoreInput {
    RawString(String),
    Image(ImageArray),
}

pub enum InputLength {
    RawString(NonZeroUsize),
    Image {
        width: NonZeroUsize,
        height: NonZeroUsize,
    },
}

#[allow(clippy::len_without_is_empty)]
impl StoreInput {
    pub fn len(&self) -> InputLength {
        match self {
            Self::Image(value) => {
                let shape = value.array.shape();
                InputLength::Image {
                    height: NonZeroUsize::new(shape[0]).unwrap(),
                    width: NonZeroUsize::new(shape[1]).unwrap(),
                }
            },
            Self::RawString(s) => InputLength::RawString(
                NonZeroUsize::new(s.len()).unwrap()),
        }
    }
}
impl fmt::Display for StoreInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString(_) => write!(f, "RawString"),
            Self::Image(_) => write!(f, "Image"),
        }
    }
}

impl From<StoreInput> for MetadataValue {
    fn from(value: StoreInput) -> Self {
        match value {
            StoreInput::Image(binary) => MetadataValue::Image(binary),
            StoreInput::RawString(s) => MetadataValue::RawString(s),
        }
    }
}

impl From<MetadataValue> for StoreInput {
    fn from(value: MetadataValue) -> Self {
        match value {
            MetadataValue::Image(binary) => StoreInput::Image(binary),
            MetadataValue::RawString(s) => StoreInput::RawString(s),
        }
    }
}
