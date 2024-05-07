use types::keyval::StoreName;

/// TODO: Move to shared rust types so library can deserialize it from the TCP response
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ServerError {
    StoreNotFound(StoreName),
    StoreAlreadyExists(StoreName),
}
