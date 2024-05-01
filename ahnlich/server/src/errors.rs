/// TODO: Move to shared rust types so library can deserialize it from the TCP response
#[derive(Debug)]
enum ServerError {
    ZeroDimensionNotAllowed,
    LockPoisonError,
}
