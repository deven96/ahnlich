package client

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	testing "testing"
	"time"

	"github.com/stretchr/testify/assert"
	"github.com/stretchr/testify/require"

	dbResponse "github.com/deven96/ahnlich/sdk/ahnlich-client-go/internal/db_response"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	transport "github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

type AhnlichClientTestSuite struct {
	client *AhnlichDBClient
	*exec.Cmd
}

// setupDatabase returns a new instance of the AhnlichClientTestSuite
func setupDatabase(t *testing.T, host string, port int) *AhnlichClientTestSuite {
	var dbClient *AhnlichDBClient
	var cmd *exec.Cmd

	t.Cleanup(func() {
		if dbClient != nil {
			dbClient.Close()
		}
		if cmd != nil {
			cmd.Process.Signal(os.Interrupt)
			cmd.Wait()
			cmd.Process.Kill() // Ensure the process is killed
		}
	})
	rootDir, err := utils.GetProjectRoot()
	require.NoError(t, err)

	tomlDir := filepath.Join(rootDir, "..", "..", "ahnlich", "Cargo.toml")
	tomlDir, err = filepath.Abs(tomlDir)
	require.NoError(t, err)

	cmd = exec.Command("cargo", "run", "--manifest-path", tomlDir, "--bin", "ahnlich-db", "run", "--port", fmt.Sprint(port))
	var outBuf, errBuf bytes.Buffer
	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	err = cmd.Start()
	require.NoError(t, err)

	// Wait for the database to start up
	maxRetries := 5
	retryInterval := 1 * time.Second

	for i := 0; i < maxRetries; i++ {
		// check if the database is running
		if cmd.ProcessState != nil {
			require.True(t, !cmd.ProcessState.Exited(), "database process exited")
			require.True(t, !cmd.ProcessState.Success(), "database process exited with success status")
		}
		// Checking stderr for the Running message as well because the database writes warnings to stderr also
		if strings.Contains(outBuf.String(), "Running") || (strings.Contains(errBuf.String(), "Running") && strings.Contains(errBuf.String(), "Finished")) {
			break
		}
		require.True(t, i < maxRetries-1, "database did not start within the expected time %v", retryInterval*time.Duration(maxRetries))
		time.Sleep(retryInterval)
	}

	// Check for any errors in stderr
	require.True(t, !strings.Contains(errBuf.String(), "error:"), "failed to start ahnlich database: %s", errBuf.String())

	config := ahnlichclientgo.LoadConfig(
		ahnlichclientgo.ConnectionConfig{
			Host:                  host,
			Port:                  port,
			InitialConnections:    1,
			MaxIdleConnections:    1,
			MaxTotalConnections:   1,
			ConnectionIdleTimeout: 5,
			ReadTimeout:           5 * time.Second,
			WriteTimeout:          5 * time.Second,
		})
	// Initialize the ahnlich database client
	cm, err := transport.NewConnectionManager(config.ConnectionConfig)
	require.NoError(t, err)

	dbClient, err = NewAhnlichDBClient(cm)
	require.NoError(t, err)

	return &AhnlichClientTestSuite{
		client: dbClient,
	}
}

func TestAhnlichClient(t *testing.T) {
	host := "127.0.0.1" // TODO: Use a random port instead of a fixed one to avoid conflicts with other tests running in parallel. Check if the port is available before running the test. Use a free port finder.
	port := 1101
	serverAddress := fmt.Sprintf("%s:%d", host, port)
	testSuite := setupDatabase(t, host, port)

	info, _ := testSuite.client.ServerInfo()
	infoResult := info[0]
	infoResultValueServerResponse := infoResult.(*dbResponse.ServerResponse__InfoServer).Value
	protoVersion, err := testSuite.client.ProtocolVersion()
	assert.NoError(t, err)
	assert.Equal(t, infoResultValueServerResponse.Version, protoVersion)
	assert.Equal(t, infoResultValueServerResponse.Address, serverAddress)

	ping, err := testSuite.client.Ping()
	assert.NoError(t, err)
	pingResult := ping[0]
	expectedPong := &dbResponse.ServerResponse__Pong{}

	assert.Equal(t, pingResult, expectedPong)

	clients, err := testSuite.client.ListClients()
	assert.NoError(t, err)
	clientsResult := clients[0]
	fmt.Println(clientsResult, "ClientList\n ")
	// 	for _,client := range clientsResultValueServerResponse {
	// 		// fmt.Println(i+1,"Client Index: ")
	// 		// fmt.Println(client.Address,"Address")
	// 		// fmt.Println(client.TimeConnected,"TimeConnected ")
	// 	}
}

// Logging
// Exceptions
// Connection Timeout
// Main Application Logic
// Config
// Tests
