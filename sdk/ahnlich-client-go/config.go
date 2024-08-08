// Description: This file contains the configuration for the client. The configuration is loaded from the environment variables. The default values are used if the environment variables are not set.
package ahnlichclientgo

import (
	"fmt"
	"time"
)

// TODO: Support for reading the configuration from environment variables and yaml files

const (
	DefaultInitialConnections     = 10
	DefaultMaxIdleConnections     = 10
	DefaultMaxTotalConnections    = 30
	DefaultConnectionIdleTimeout  = 5 * time.Minute
	DefaultReadTimeout            = 10 * time.Second
	DefaultWriteTimeout           = 10 * time.Second
	DefaultBackoffMaxElapsedTime  = 2 * time.Minute
	DefaultBackoffInitialInterval = 1 * time.Second
	DefaultBackoffMaxInterval     = DefaultBackoffInitialInterval * 2
	DefaultKeepAlivePeriod        = 30 * time.Second
	VersionFile                   = "/VERSION"
)

type Config struct {
	ConnectionConfig
	ClientConfig
}

type ClientConfig struct {
	// Add any client specific configuration here
}

// ConnectionConfig holds the configuration for the connection pool
type ConnectionConfig struct {
	InitialConnections     int           // Initial number of connections to be created
	MaxIdleConnections     int           // Maximum number of idle connections to be maintained
	MaxTotalConnections    int           // Maximum number of total connections to be maintained
	ConnectionIdleTimeout  time.Duration // Time after which the connection is closed if it is idle
	ReadTimeout            time.Duration // Read timeout for the connection
	WriteTimeout           time.Duration // Write timeout for the connection
	ServerAddress          string        // Server address in the format "host:port"
	Host                   string        // Hostname of the server
	Port                   int           // Port of the server
	BackoffMaxElapsedTime  time.Duration // Maximum time to wait for the backoff in connection retry mechanism
	BackoffInitialInterval time.Duration // Initial interval for the backoff in connection retry mechanism
	BackoffMaxInterval     time.Duration // Maximum interval for the backoff in connection retry mechanism
	SetKeepAlive           bool          // Set TCP keep-alive
	KeepAlivePeriod        time.Duration // Keep-alive period for the TCP connection
}

func LoadConfig(connCfg ConnectionConfig) Config {
	if connCfg.InitialConnections <= 0 {
		connCfg.InitialConnections = DefaultInitialConnections
	}
	if connCfg.MaxIdleConnections <= 0 {
		connCfg.MaxIdleConnections = DefaultMaxIdleConnections
	}
	if connCfg.MaxTotalConnections <= 0 {
		connCfg.MaxTotalConnections = DefaultMaxTotalConnections
	}
	if connCfg.ConnectionIdleTimeout <= 0 {
		connCfg.ConnectionIdleTimeout = DefaultConnectionIdleTimeout
	}
	if connCfg.ReadTimeout <= 0 {
		connCfg.ReadTimeout = DefaultReadTimeout
	}
	if connCfg.WriteTimeout <= 0 {
		connCfg.WriteTimeout = DefaultWriteTimeout
	}
	if (connCfg.Host == "" || connCfg.Port == 0) && connCfg.ServerAddress == "" {
		panic("(Host and Port) or ServerAddress must be provided in the ConnectionConfig")
	}
	if connCfg.ServerAddress == "" {
		connCfg.ServerAddress = fmt.Sprintf("%s:%d", connCfg.Host, connCfg.Port)
	}
	if connCfg.BackoffMaxElapsedTime <= 0 {
		connCfg.BackoffMaxElapsedTime = DefaultBackoffMaxElapsedTime
	}

	if connCfg.BackoffInitialInterval <= 0 {
		connCfg.BackoffInitialInterval = DefaultBackoffInitialInterval
	}

	if connCfg.BackoffMaxInterval <= 0 {
		connCfg.BackoffMaxInterval = DefaultBackoffMaxInterval
	}

	if connCfg.SetKeepAlive && connCfg.KeepAlivePeriod <= 0 {
		connCfg.KeepAlivePeriod = DefaultKeepAlivePeriod
	}

	// Insert the default client configuration into the config
	clientCfg := ClientConfig{}
	return Config{
		ConnectionConfig: connCfg,
		ClientConfig:     clientCfg,
	}
}
