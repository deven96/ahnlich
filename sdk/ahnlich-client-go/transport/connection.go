package transport

import (
	"fmt"
	"net"
	"time"

	backoff "github.com/cenkalti/backoff/v4"
	"github.com/silenceper/pool"
	"github.com/sirupsen/logrus"

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

	var p pool.Pool
	var err error

	// Retry mechanism with backoff
	operation := func() error {
		p, err = pool.NewChannelPool(&pool.Config{
			InitialCap: cfg.InitialConnections,
			MaxIdle:    cfg.MaxIdleConnections,
			MaxCap:     cfg.MaxTotalConnections,
			Factory: func() (interface{}, error) {
				conn, err := net.Dial("tcp", cfg.ServerAddress)
				if err != nil {
					return nil, err
				}
				// Set TCP keep-alive
				if tcpConn, ok := conn.(*net.TCPConn); ok {
					tcpConn.SetKeepAlive(cfg.SetKeepAlive)
					tcpConn.SetKeepAlivePeriod(cfg.KeepAlivePeriod)
					// tcpConn.SetWriteBuffer(1024)
					// tcpConn.SetReadBuffer(1024)
				}
				return conn, nil
			},
			Close: func(v interface{}) error {
				return v.(net.Conn).Close()
			},
			IdleTimeout: cfg.ConnectionIdleTimeout,
			//TODO: Better way to check if the connection is still alive
			Ping: func(v interface{}) error {
				conn := v.(net.Conn)
				_, err := conn.Write([]byte("PING\n"))
				// if err != nil {
				// 	return err
				// }
				// readBuf := make([]byte, 0)
				// _,err = conn.Read(readBuf)
				return err
			},
		})
		if err != nil {
			logrus.Error("Unable to create connection pool", "error", err)
		}
		return err
	}

	// Use exponential backoff with a max elapsed time
	backoffConfig := backoff.NewExponentialBackOff(
		backoff.WithInitialInterval(cfg.BackoffInitialInterval),
		backoff.WithMaxInterval(cfg.BackoffMaxInterval),
		// backoff.WithMultiplier(cfg.BackoffMultiplier),
		// backoff.WithRandomizationFactor(cfg.BackoffRandomizationFactor),
		backoff.WithMaxElapsedTime(cfg.BackoffMaxElapsedTime),
	)

	err = backoff.Retry(operation, backoffConfig)
	if err != nil {
		err = fmt.Errorf("failed to create connection pool after retries error %v", err)
		return nil, &utils.AhnlichClientException{Message: err.Error()} // Convert to AhnlichClientConnectionException
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
