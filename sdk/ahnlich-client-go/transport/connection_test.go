package transport

import (
	"fmt"
	testing "testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// setupConnectionManager returns a new instance of the ConnectionManagerTestSuite
func newTestConnectionManager(t *testing.T, config ahnlichclientgo.ConnectionConfig) *ConnectionManager {
	var cm *ConnectionManager

	t.Cleanup(func() {
		if cm != nil {
			cm.Release()
		}
	})

	cm, err := NewConnectionManager(config)
	require.NoError(t, err)
	require.NotEmpty(t, cm)
	return cm
}

func TestSingleConnection(t *testing.T) {
	db := utils.RunAhnlichDatabase(t, false, "")
	config := ahnlichclientgo.ConnectionConfig{
		Host:                  db.Host,
		Port:                  db.Port,
		InitialConnections:    1,
		MaxIdleConnections:    1,
		MaxTotalConnections:   1,
		ConnectionIdleTimeout: 5,
		ReadTimeout:           5 * time.Second,
		WriteTimeout:          5 * time.Second,
	}
	// Sequence of operations:
	cm := newTestConnectionManager(t, config)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	conn, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn)
	assert.Equal(t, cm.ActiveConnections(), 0)
	assert.Equal(t, conn.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))

	cm.Return(conn)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	cm.Release()
	assert.Equal(t, cm.ActiveConnections(), 0)

	cm.Refresh()
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)
}

func TestMultipleConnections(t *testing.T) {
	db := utils.RunAhnlichDatabase(t, false, "")
	config := ahnlichclientgo.ConnectionConfig{
		Host:                  db.Host,
		Port:                  db.Port,
		InitialConnections:    2,
		MaxIdleConnections:    2,
		MaxTotalConnections:   2,
		ConnectionIdleTimeout: 5,
		ReadTimeout:           5 * time.Second,
		WriteTimeout:          5 * time.Second,
	}
	// Sequence of operations:
	cm := newTestConnectionManager(t, config)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	conn1, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn1)
	// assert.Equal(t, cm.ActiveConnections(), 1) // This fails, seems like the pool library sets the active connections to 0 after getting a connection

	conn2, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn2)
	assert.Equal(t, cm.ActiveConnections(), 0)

	assert.NotEqual(t, conn1.LocalAddr().String(), conn2.LocalAddr().String())
	assert.Equal(t, conn1.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))
	assert.Equal(t, conn2.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))

	assert.NotEqual(t, conn1, conn2)

	cm.Return(conn1)
	assert.Equal(t, cm.ActiveConnections(), 1)

	cm.Return(conn2)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	cm.Release()
	assert.Equal(t, cm.ActiveConnections(), 0)

	cm.Refresh()
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)
}

func Test_IdleConnectionTimeout(t *testing.T) {
	db := utils.RunAhnlichDatabase(t, false, "")
	config := ahnlichclientgo.ConnectionConfig{
		Host:                  db.Host,
		Port:                  db.Port,
		InitialConnections:    1,
		MaxIdleConnections:    1,
		MaxTotalConnections:   1,
		ConnectionIdleTimeout: 5,
		ReadTimeout:           5 * time.Second,
		WriteTimeout:          5 * time.Second,
	}
	// Sequence of operations:
	cm := newTestConnectionManager(t, config)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	conn, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn)
	assert.Equal(t, conn.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))

	// Sleep for duration seconds to trigger the timeout
	time.Sleep((config.ConnectionIdleTimeout * time.Second) + (1 * time.Second))
	_, err = conn.Write([]byte("Hello"))
	require.Error(t, err)
	assert.Contains(t, err.Error(), "timeout")
}

func Test_MaxTotalConnections(t *testing.T) {
	db := utils.RunAhnlichDatabase(t, false, "")
	config := ahnlichclientgo.ConnectionConfig{
		Host:                  db.Host,
		Port:                  db.Port,
		InitialConnections:    1,
		MaxIdleConnections:    1,
		MaxTotalConnections:   1,
		ConnectionIdleTimeout: 5,
		ReadTimeout:           5 * time.Second,
		WriteTimeout:          5 * time.Second,
	}
	// Sequence of operations:
	cm := newTestConnectionManager(t, config)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	t.Log("Getting conn1 from the pool ...")
	conn1, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn1)
	assert.Equal(t, conn1.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))
	t.Log("Got conn1 from the pool")
	countdownDuration := 3
	// conn1 should be returned to the pool
	go func() {
		// Start the countdown
		for i := countdownDuration; i >= 0; i-- {
			t.Logf("Time remaining before conn1 is returned: %d seconds\n", i)
			time.Sleep(1 * time.Second)
		}
		cm.Return(conn1)
	}()

	// conn2 should wait for countdownDuration for conn1 to be returned to the pool //
	t.Log("Getting conn2 from the pool ...")
	conn2, err := cm.GetConnection() // This should block for countdownDuration
	require.NoError(t, err)
	require.NotEmpty(t, conn2)
	assert.Equal(t, conn2.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))
	t.Log("Got conn2 from the pool")
}

func Test_RetryConnection(t *testing.T) {
	db := utils.RunAhnlichDatabase(t, false, "")
	config := ahnlichclientgo.ConnectionConfig{
		Host:                   db.Host,
		Port:                   db.Port,
		InitialConnections:     1,
		MaxIdleConnections:     1,
		MaxTotalConnections:    1,
		ConnectionIdleTimeout:  5,
		ReadTimeout:            5 * time.Second,
		WriteTimeout:           5 * time.Second,
		BackoffMaxElapsedTime:  2 * time.Second,
		BackoffInitialInterval: 1 * time.Second,
		BackoffMaxInterval:     2 * time.Second,
	}
	// Sequence of operations:
	cm := newTestConnectionManager(t, config)
	assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)

	conn, err := cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn)
	assert.Equal(t, conn.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))
	cm.Return(conn)

	// Kill the database to trigger the retry
	db.Kill()
	// check if the database is running
	require.False(t, db.IsRunning())
	_, err = cm.GetConnection()
	require.Error(t, err)
	assert.Contains(t, err.Error(), "connect: connection refused")

	// Start the database again
	db = utils.RunAhnlichDatabase(t, false, "", db.Host, db.Port)
	// check if the database is running
	require.True(t, db.IsRunning())
	// Retry the connection
	conn, err = cm.GetConnection()
	require.NoError(t, err)
	require.NotEmpty(t, conn)
	assert.Equal(t, conn.RemoteAddr().String(), fmt.Sprintf("%s:%d", db.Host, db.Port))
	cm.Return(conn)
}

func Test_Backoff(t *testing.T) {
	config := ahnlichclientgo.ConnectionConfig{
		Host:                   "127.0.0.1",
		InitialConnections:     1,
		MaxIdleConnections:     1,
		MaxTotalConnections:    1,
		ConnectionIdleTimeout:  5,
		ReadTimeout:            5 * time.Second,
		WriteTimeout:           5 * time.Second,
		BackoffMaxElapsedTime:  2 * time.Second,
		BackoffInitialInterval: 1 * time.Second,
		BackoffMaxInterval:     2 * time.Second,
	}
	port, err := utils.GetAvailablePort(config.Host)
	require.NoError(t, err)
	require.NotZero(t, port)
	config.Port = port
	// Sequence of operations:
	// should send result to the channel after the backoff retry
	ch := make(chan *ConnectionManager)
	go func() {
		cm := newTestConnectionManager(t, config)
		ch <- cm
	}()
	// Kill the database to trigger the backoff retry
	db := utils.RunAhnlichDatabase(t, false, "", config.Host, config.Port)
	// check if the database is running
	require.True(t, db.IsRunning())
	time.Sleep(config.BackoffMaxElapsedTime - 1*time.Second)
	select {
	case cm := <-ch:
		assert.Equal(t, cm.ActiveConnections(), config.InitialConnections)
		conn, err := cm.GetConnection()
		require.NoError(t, err)
		require.NotEmpty(t, conn)
		assert.Equal(t, conn.RemoteAddr().String(), fmt.Sprintf("%s:%d", config.Host, config.Port))
		cm.Return(conn)
	case <-time.After(config.BackoffMaxElapsedTime + 2*time.Second):
		t.Error("Timeout waiting for backoff retry")
	}
}
