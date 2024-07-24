package transport

import (
	"fmt"
	"log/slog"
	"net"
	"time"

	"github.com/silenceper/pool"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// ConnectionManager manages the TCP connection pool
type ConnectionManager struct {
	connectionPool pool.Pool
	cfg            ahnlichclientgo.ConnectionConfig
}

func createConnectionPool(cfg ahnlichclientgo.ConnectionConfig) (pool.Pool, error) {
	if cfg.ServerAddress == "" && (cfg.Host == "" || cfg.Port == 0) {
		return nil, &utils.AhnlichClientException{Message: "Server address is empty"}
	} else if cfg.ServerAddress == "" {
		cfg.ServerAddress = cfg.Host + ":" + fmt.Sprint(cfg.Port)
	}

	p, err := pool.NewChannelPool(&pool.Config{
		InitialCap: cfg.InitialConnections,
		MaxIdle:    cfg.MaxIdleConnections,
		MaxCap:     cfg.MaxTotalConnections,
		Factory: func() (interface{}, error) {
			return net.Dial("tcp", cfg.ServerAddress)
		},
		Close: func(v interface{}) error {
			return v.(net.Conn).Close()
		},
		IdleTimeout: cfg.ConnectionIdleTimeout,
		Ping: func(v interface{}) error {
			_, err := v.(net.Conn).Write([]byte("PING\n"))
			return err
		},
	})
	if err != nil {
		slog.Error("Unable to create connection pool", "error", err)
		return nil, &utils.AhnlichClientException{Message: "Unable to create connection pool"} // Convert to AhnlichClientConnectionException
	}

	return p, nil
}

func NewConnectionManager(cfg ahnlichclientgo.ConnectionConfig) (*ConnectionManager, error) {
	p, err := createConnectionPool(cfg)
	if err != nil {
		return nil, err
	}
	return &ConnectionManager{
		connectionPool: p,
		cfg:            cfg,
	}, nil
}

// GetConnection retrieves a connection from the pool
func (cm *ConnectionManager) GetConnection() (net.Conn, error) {
	connInterface, err := cm.connectionPool.Get()
	if err != nil {
		return nil, err
	}
	conn := connInterface.(net.Conn)
	err = conn.SetReadDeadline(time.Now().Add(cm.cfg.ReadTimeout))
	if err != nil {
		return nil, err
	}
	err = conn.SetWriteDeadline(time.Now().Add(cm.cfg.WriteTimeout))
	if err != nil {
		return nil, err
	}
	return conn, nil
}

// Return returns a connection back to the pool after use
func (cm *ConnectionManager) Return(conn net.Conn) {
	cm.connectionPool.Put(conn)
}

// Release closes all connections in the pool
func (cm *ConnectionManager) Release() {
	cm.connectionPool.Release()
}

// ActiveConnections returns the number of active connections in the pool
func (cm *ConnectionManager) ActiveConnections() int {
	return cm.connectionPool.Len()
}

func (cm *ConnectionManager) Refresh() error {
	cm.connectionPool.Release()
	newPool, err := createConnectionPool(cm.cfg)
	if err != nil {
		return err
	}
	cm.connectionPool = newPool
	return nil
}
