use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;
use tonic::transport::{Identity, ServerTlsConfig};
use tonic::{Request, Status};

pub const AUTH_HEADER: &str = "authorization";

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Failed to read auth config file: {0}")]
    ConfigReadError(String),

    #[error("Failed to parse auth config: {0}")]
    ConfigParseError(String),

    #[error("Missing authorization header")]
    MissingAuthHeader,

    #[error("Invalid authorization format, expected 'Bearer username:api_key'")]
    InvalidAuthFormat,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("API key too short, minimum length: {0}")]
    KeyTooShort(usize),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub users: HashMap<String, String>,
    #[serde(default = "default_security")]
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    #[serde(default = "default_min_key_length")]
    pub min_key_length: usize,
}

fn default_security() -> SecurityConfig {
    SecurityConfig {
        min_key_length: default_min_key_length(),
    }
}

fn default_min_key_length() -> usize {
    16
}

impl AuthConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, AuthError> {
        let content =
            fs::read_to_string(path).map_err(|e| AuthError::ConfigReadError(e.to_string()))?;

        toml::from_str(&content).map_err(|e| AuthError::ConfigParseError(e.to_string()))
    }

    pub fn validate_credentials(&self, username: &str, api_key: &str) -> Result<(), AuthError> {
        if api_key.len() < self.security.min_key_length {
            return Err(AuthError::KeyTooShort(self.security.min_key_length));
        }

        let stored_hash = self
            .users
            .get(username)
            .ok_or(AuthError::InvalidCredentials)?;

        let provided_hash = hash_api_key(api_key);

        if constant_time_compare(stored_hash, &provided_hash) {
            Ok(())
        } else {
            Err(AuthError::InvalidCredentials)
        }
    }
}

pub fn hash_api_key(api_key: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(api_key.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn constant_time_compare(a: &str, b: &str) -> bool {
    if a.len() != b.len() {
        return false;
    }

    a.as_bytes()
        .iter()
        .zip(b.as_bytes().iter())
        .fold(0u8, |acc, (a, b)| acc | (a ^ b))
        == 0
}

pub fn parse_bearer_token(auth_value: &str) -> Result<(String, String), AuthError> {
    let parts: Vec<&str> = auth_value.split_whitespace().collect();

    if parts.len() != 2 || parts[0] != "Bearer" {
        return Err(AuthError::InvalidAuthFormat);
    }

    let credentials: Vec<&str> = parts[1].split(':').collect();
    if credentials.len() != 2 {
        return Err(AuthError::InvalidAuthFormat);
    }

    Ok((credentials[0].to_string(), credentials[1].to_string()))
}

/// Load TLS configuration from certificate and key files
pub fn load_tls_config(
    cert_path: &PathBuf,
    key_path: &PathBuf,
) -> Result<ServerTlsConfig, AuthError> {
    let cert = fs::read_to_string(cert_path)
        .map_err(|e| AuthError::ConfigReadError(format!("Failed to read cert file: {}", e)))?;
    let key = fs::read_to_string(key_path)
        .map_err(|e| AuthError::ConfigReadError(format!("Failed to read key file: {}", e)))?;

    let identity = Identity::from_pem(cert, key);

    Ok(ServerTlsConfig::new().identity(identity))
}

/// gRPC interceptor for validating API key authentication
#[derive(Clone)]
pub struct AuthInterceptor {
    config: Arc<AuthConfig>,
}

impl AuthInterceptor {
    pub fn new(config: Arc<AuthConfig>) -> Self {
        Self { config }
    }

    #[allow(clippy::result_large_err)] // tonic::Status size is fixed by the tonic API contract
    pub fn intercept<T>(&self, request: Request<T>) -> Result<Request<T>, Status> {
        let metadata = request.metadata();

        let auth_header = metadata
            .get(AUTH_HEADER)
            .ok_or_else(|| Status::unauthenticated(AuthError::MissingAuthHeader.to_string()))?
            .to_str()
            .map_err(|_| {
                Status::invalid_argument("Authorization header contains invalid characters")
            })?;

        let (username, api_key) = parse_bearer_token(auth_header).map_err(|e| match e {
            AuthError::InvalidAuthFormat => Status::invalid_argument(e.to_string()),
            _ => Status::unauthenticated(e.to_string()),
        })?;

        self.config
            .validate_credentials(&username, &api_key)
            .map_err(|e| match e {
                AuthError::KeyTooShort(_) => Status::invalid_argument(e.to_string()),
                AuthError::InvalidCredentials => Status::unauthenticated(e.to_string()),
                _ => Status::internal(format!("Authentication error: {}", e)),
            })?;

        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_api_key() {
        let key = "my_secret_key_123";
        let hash1 = hash_api_key(key);
        let hash2 = hash_api_key(key);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_constant_time_compare() {
        assert!(constant_time_compare("abc", "abc"));
        assert!(!constant_time_compare("abc", "abd"));
        assert!(!constant_time_compare("abc", "abcd"));
    }

    #[test]
    fn test_parse_bearer_token_valid() {
        let result = parse_bearer_token("Bearer alice:secret123");
        assert!(result.is_ok());
        let (username, key) = result.unwrap();
        assert_eq!(username, "alice");
        assert_eq!(key, "secret123");
    }

    #[test]
    fn test_parse_bearer_token_invalid_format() {
        assert!(parse_bearer_token("alice:secret123").is_err());
        assert!(parse_bearer_token("Bearer alice").is_err());
        assert!(parse_bearer_token("Basic alice:secret123").is_err());
    }

    #[test]
    fn test_validate_credentials() {
        let mut users = HashMap::new();
        users.insert("alice".to_string(), hash_api_key("my_super_secret_key"));

        let config = AuthConfig {
            users,
            security: SecurityConfig { min_key_length: 16 },
        };

        assert!(
            config
                .validate_credentials("alice", "my_super_secret_key")
                .is_ok()
        );
        assert!(
            config
                .validate_credentials("bob", "my_super_secret_key")
                .is_err()
        );
        assert!(config.validate_credentials("alice", "wrong_key").is_err());
        assert!(matches!(
            config.validate_credentials("alice", "short"),
            Err(AuthError::KeyTooShort(16))
        ));
    }

    #[test]
    fn test_auth_interceptor_valid() {
        let mut users = HashMap::new();
        users.insert("alice".to_string(), hash_api_key("my_super_secret_key"));

        let config = Arc::new(AuthConfig {
            users,
            security: SecurityConfig { min_key_length: 16 },
        });

        let interceptor = AuthInterceptor::new(config);
        let mut request = Request::new(());
        request.metadata_mut().insert(
            AUTH_HEADER,
            "Bearer alice:my_super_secret_key".parse().unwrap(),
        );

        assert!(interceptor.intercept(request).is_ok());
    }

    #[test]
    fn test_auth_interceptor_missing_header() {
        let config = Arc::new(AuthConfig {
            users: HashMap::new(),
            security: SecurityConfig { min_key_length: 16 },
        });

        let interceptor = AuthInterceptor::new(config);
        let request = Request::new(());

        let result = interceptor.intercept(request);
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }

    #[test]
    fn test_auth_interceptor_invalid_format() {
        let config = Arc::new(AuthConfig {
            users: HashMap::new(),
            security: SecurityConfig { min_key_length: 16 },
        });

        let interceptor = AuthInterceptor::new(config);
        let mut request = Request::new(());
        request
            .metadata_mut()
            .insert(AUTH_HEADER, "alice:key".parse().unwrap());

        let result = interceptor.intercept(request);
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[test]
    fn test_auth_interceptor_invalid_credentials() {
        let mut users = HashMap::new();
        users.insert("alice".to_string(), hash_api_key("my_super_secret_key"));

        let config = Arc::new(AuthConfig {
            users,
            security: SecurityConfig { min_key_length: 16 },
        });

        let interceptor = AuthInterceptor::new(config);
        let mut request = Request::new(());
        request.metadata_mut().insert(
            AUTH_HEADER,
            "Bearer alice:wrong_password_here".parse().unwrap(),
        );

        let result = interceptor.intercept(request);
        assert!(result.is_err());
        let status = result.unwrap_err();
        assert_eq!(status.code(), tonic::Code::Unauthenticated);
    }
}
