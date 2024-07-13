package transport

import (
	"net"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	"github.com/silenceper/pool"
)

// ConnectionManager manages the TCP connection pool
type ConnectionManager struct {
    ConnectionPool pool.Pool
	Cfg ahnlichclientgo.Config
}

func NewConnectionManager( cfg ahnlichclientgo.Config) (*ConnectionManager,error) {
	factory := func() (interface{}, error) {
        return net.Dial("tcp", cfg.ServerAddress)
    }

    close := func(v interface{}) error {
        return v.(net.Conn).Close()
    }

    p, err := pool.NewChannelPool(&pool.Config{
        InitialCap:  cfg.InitialConnections,
        MaxIdle:     cfg.MaxIdleConnections,
        MaxCap:      cfg.MaxTotalConnections,
        Factory:     factory,
        Close:       close,
        IdleTimeout: cfg.ConnectionIdleTimeout,
    })
    if err != nil {
        return nil, err
    }

	return &ConnectionManager{
		ConnectionPool: p,
		Cfg: cfg,
}, nil
}

// GetConnection retrieves a connection from the pool
func (cm *ConnectionManager) GetConnection() (net.Conn, error) {
    conn, err := cm.ConnectionPool.Get()
    if err != nil {
        return nil, err
    }
    return conn.(net.Conn), nil
}


// Return returns a connection back to the pool after use
func (cm *ConnectionManager) Return(conn net.Conn) {
    cm.ConnectionPool.Put(conn)
}

// Release closes all connections in the pool
func (cm *ConnectionManager) Release() {
    cm.ConnectionPool.Release()
}

// ActiveConnections returns the number of active connections in the pool
func (cm *ConnectionManager) ActiveConnections() int {
    return cm.ConnectionPool.Len()
}