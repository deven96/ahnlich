---
title: Authentication
sidebar_position: 5
---

# Authentication

Ahnlich supports API key authentication with TLS encryption for secure production deployments.

## Overview

Authentication in Ahnlich uses:

- **Bearer token authentication** via the `authorization` header
- **TLS encryption** for secure transport (required when auth is enabled)
- **SHA-256 hashed API keys** stored in a TOML config file

## Server Configuration

### Enable Authentication

Start the server with authentication enabled:

```bash
ahnlich-db run \
  --host 0.0.0.0 \
  --enable-auth \
  --auth-config /path/to/auth.toml \
  --tls-cert /path/to/server.crt \
  --tls-key /path/to/server.key
```

The same flags apply to `ahnlich-ai`:

```bash
ahnlich-ai run \
  --host 0.0.0.0 \
  --db-host ahnlich_db \
  --enable-auth \
  --auth-config /path/to/auth.toml \
  --tls-cert /path/to/server.crt \
  --tls-key /path/to/server.key
```

### CLI Flags

| Flag | Description |
|------|-------------|
| `--enable-auth` | Enable authentication (requires TLS) |
| `--auth-config <path>` | Path to authentication config file (TOML format) |
| `--tls-cert <path>` | Path to TLS certificate file (PEM format) |
| `--tls-key <path>` | Path to TLS private key file (PEM format) |

## Auth Config File

Create a TOML file with usernames and SHA-256 hashed API keys:

```toml
# auth.toml

[users]
alice = "5e884898da28047d55d0cb6af8e6d0df01c52e4b0c3c75c4cc11e8f9e6b0e4a2"
bob = "a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3"

[security]
min_key_length = 16
```

### Generate API Key Hashes

Use any SHA-256 tool to hash your API keys:

```bash
# Using openssl
echo -n "my_super_secret_key" | openssl dgst -sha256

# Using sha256sum
echo -n "my_super_secret_key" | sha256sum
```

The output hash (without the filename suffix) goes in the config file.

### Security Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `min_key_length` | 16 | Minimum API key length (before hashing) |

## Client Authentication

### Header Format

All authenticated requests must include the `authorization` header:

```
authorization: Bearer <username>:<api_key>
```

The API key is sent in plain text over TLS. The server hashes it and compares against the stored hash.

### Node.js

```ts
import * as fs from "fs";
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369", {
  caCert: fs.readFileSync("ca.crt"),
  auth: { username: "alice", apiKey: "my_super_secret_key" },
});
```

The `caCert` option provides the CA certificate for TLS verification. This is required when the server uses a self-signed certificate or a private CA.

### Python

```python
import asyncio
import ssl
from grpclib.client import Channel

async def create_authenticated_client():
    # Create SSL context with CA certificate
    ssl_context = ssl.create_default_context()
    ssl_context.load_verify_locations("ca.crt")
    
    # Create channel with TLS
    channel = Channel(
        host="127.0.0.1",
        port=1369,
        ssl=ssl_context
    )
    
    # Add authorization metadata to requests
    metadata = {"authorization": "Bearer alice:my_super_secret_key"}
    
    return channel, metadata
```

### Go

```go
import (
    "context"
    "crypto/tls"
    "crypto/x509"
    "os"
    
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials"
    "google.golang.org/grpc/metadata"
)

func createAuthenticatedClient(ctx context.Context) (*grpc.ClientConn, error) {
    // Load CA certificate
    caCert, err := os.ReadFile("ca.crt")
    if err != nil {
        return nil, err
    }
    
    certPool := x509.NewCertPool()
    certPool.AppendCertsFromPEM(caCert)
    
    // Create TLS credentials
    tlsConfig := &tls.Config{RootCAs: certPool}
    creds := credentials.NewTLS(tlsConfig)
    
    // Connect with TLS
    conn, err := grpc.DialContext(ctx, "127.0.0.1:1369",
        grpc.WithTransportCredentials(creds),
        grpc.WithBlock(),
    )
    if err != nil {
        return nil, err
    }
    
    return conn, nil
}

// Add auth header to context for each request
func withAuth(ctx context.Context) context.Context {
    return metadata.AppendToOutgoingContext(ctx,
        "authorization", "Bearer alice:my_super_secret_key",
    )
}
```

### Rust

```rust
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tonic::Request;

async fn create_authenticated_client() -> Result<Channel, Box<dyn std::error::Error>> {
    let ca_cert = std::fs::read_to_string("ca.crt")?;
    let ca = Certificate::from_pem(ca_cert);
    
    let tls = ClientTlsConfig::new().ca_certificate(ca);
    
    let channel = Channel::from_static("https://127.0.0.1:1369")
        .tls_config(tls)?
        .connect()
        .await?;
    
    Ok(channel)
}

// Add auth header to requests
fn with_auth<T>(mut request: Request<T>) -> Request<T> {
    request.metadata_mut().insert(
        "authorization",
        "Bearer alice:my_super_secret_key".parse().unwrap(),
    );
    request
}
```

## Docker Compose with Auth

```yaml
version: "3.8"

services:
  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: >
      ahnlich-db run --host 0.0.0.0
      --enable-auth
      --auth-config /etc/ahnlich/auth.toml
      --tls-cert /etc/ahnlich/server.crt
      --tls-key /etc/ahnlich/server.key
      --enable-persistence
      --persist-location /root/.ahnlich/data/db.dat
    ports:
      - "1369:1369"
    volumes:
      - ./data:/root/.ahnlich/data
      - ./certs:/etc/ahnlich:ro
```

## TLS Certificate Setup

### Generate Self-Signed Certificates

For development or internal use:

```bash
# Generate CA key and certificate
openssl genrsa -out ca.key 4096
openssl req -new -x509 -days 365 -key ca.key -out ca.crt \
  -subj "/CN=Ahnlich CA"

# Generate server key and CSR
openssl genrsa -out server.key 2048
openssl req -new -key server.key -out server.csr \
  -subj "/CN=localhost"

# Sign server certificate with CA
openssl x509 -req -days 365 -in server.csr -CA ca.crt -CAkey ca.key \
  -CAcreateserial -out server.crt

# Clean up CSR
rm server.csr
```

### Production Certificates

For production, use certificates from a trusted CA (Let's Encrypt, DigiCert, etc.) or your organization's internal CA.

## Error Responses

| Error | Cause |
|-------|-------|
| `Missing authorization header` | No `authorization` header provided |
| `Invalid authorization format` | Header doesn't match `Bearer username:api_key` |
| `Invalid credentials` | Username not found or API key doesn't match |
| `API key too short` | Key shorter than `min_key_length` |

## Security Best Practices

1. **Use strong API keys**: At least 32 characters with mixed case, numbers, and symbols
2. **Rotate keys periodically**: Update the auth config and restart the server
3. **Protect the auth config**: Restrict file permissions (`chmod 600 auth.toml`)
4. **Use trusted TLS certificates**: Avoid self-signed certs in production
5. **Monitor failed auth attempts**: Check server logs for unauthorized access attempts
