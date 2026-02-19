package ahnlichgotest

import (
	"crypto/ecdsa"
	"crypto/elliptic"
	"crypto/rand"
	"crypto/sha256"
	"crypto/tls"
	"crypto/x509"
	"crypto/x509/pkix"
	"encoding/hex"
	"encoding/pem"
	"fmt"
	"math/big"
	"net"
	"os"
	"path/filepath"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
	"google.golang.org/grpc/credentials"
)

// AuthConfig holds paths to generated TLS and auth config files.
type AuthConfig struct {
	CertPath       string
	KeyPath        string
	AuthConfigPath string
	CertPEM        []byte
	TmpDir         string
}

// GenerateTestTLS generates a self-signed TLS certificate and key into a temp directory.
// Returns an AuthConfig with file paths and the raw cert PEM (for client trust).
func GenerateTestTLS(t *testing.T) *AuthConfig {
	t.Helper()

	tmpDir, err := os.MkdirTemp("", "ahnlich-auth-*")
	require.NoError(t, err)
	t.Cleanup(func() { os.RemoveAll(tmpDir) })

	key, err := ecdsa.GenerateKey(elliptic.P256(), rand.Reader)
	require.NoError(t, err)

	template := &x509.Certificate{
		SerialNumber: big.NewInt(1),
		Subject:      pkix.Name{CommonName: "localhost"},
		NotBefore:    time.Now().Add(-time.Hour),
		NotAfter:     time.Now().Add(24 * time.Hour),
		KeyUsage:     x509.KeyUsageDigitalSignature,
		ExtKeyUsage:  []x509.ExtKeyUsage{x509.ExtKeyUsageServerAuth},
		IPAddresses:  []net.IP{net.ParseIP("127.0.0.1")},
		DNSNames:     []string{"localhost"},
	}

	certDER, err := x509.CreateCertificate(rand.Reader, template, template, &key.PublicKey, key)
	require.NoError(t, err)

	certPEM := pem.EncodeToMemory(&pem.Block{Type: "CERTIFICATE", Bytes: certDER})

	keyDER, err := x509.MarshalECPrivateKey(key)
	require.NoError(t, err)
	keyPEM := pem.EncodeToMemory(&pem.Block{Type: "EC PRIVATE KEY", Bytes: keyDER})

	certPath := filepath.Join(tmpDir, "server.crt")
	keyPath := filepath.Join(tmpDir, "server.key")
	require.NoError(t, os.WriteFile(certPath, certPEM, 0600))
	require.NoError(t, os.WriteFile(keyPath, keyPEM, 0600))

	return &AuthConfig{
		CertPath: certPath,
		KeyPath:  keyPath,
		CertPEM:  certPEM,
		TmpDir:   tmpDir,
	}
}

// WriteAuthConfig writes a TOML auth config file with the given users (username -> plaintext api_key).
// Keys are SHA256-hashed before writing.
func WriteAuthConfig(t *testing.T, cfg *AuthConfig, users map[string]string) {
	t.Helper()
	content := "[users]\n"
	for username, apiKey := range users {
		sum := sha256.Sum256([]byte(apiKey))
		content += fmt.Sprintf("%s = \"%s\"\n", username, hex.EncodeToString(sum[:]))
	}
	content += "\n[security]\nmin_key_length = 8\n"
	path := filepath.Join(cfg.TmpDir, "auth.toml")
	require.NoError(t, os.WriteFile(path, []byte(content), 0600))
	cfg.AuthConfigPath = path
}

// ClientTLSCredentials returns gRPC TLS credentials that trust only the given cert PEM.
func ClientTLSCredentials(t *testing.T, certPEM []byte) credentials.TransportCredentials {
	t.Helper()
	pool := x509.NewCertPool()
	require.True(t, pool.AppendCertsFromPEM(certPEM), "failed to add cert to pool")
	return credentials.NewTLS(&tls.Config{
		RootCAs:    pool,
		ServerName: "localhost",
	})
}

// AuthFlag is an OptionalFlags implementation that wires --enable-auth, --auth-config, --tls-cert, --tls-key.
type AuthFlag struct {
	Cfg *AuthConfig
	baseFlag
}

func (a *AuthFlag) parseArgs() ([]string, error) {
	a.Flags = []string{
		"--enable-auth",
		"--auth-config", a.Cfg.AuthConfigPath,
		"--tls-cert", a.Cfg.CertPath,
		"--tls-key", a.Cfg.KeyPath,
	}
	return a.Flags, nil
}
