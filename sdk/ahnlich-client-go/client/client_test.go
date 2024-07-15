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

	"github.com/stretchr/testify/require"

	ahnlichclientgo "github.com/deven96/ahnlich/sdk/ahnlich-client-go"
	transport "github.com/deven96/ahnlich/sdk/ahnlich-client-go/transport"
	"github.com/deven96/ahnlich/sdk/ahnlich-client-go/utils"
)

// store_payload_no_predicates = {
//     "store_name": "Diretnan Station",
//     "dimension": 5,
//     "error_if_exists": True,
// }

//	store_payload_with_predicates = {
//	    "store_name": "Diretnan Predication",
//	    "dimension": 5,
//	    "error_if_exists": True,
//	    "create_predicates": ["is_tyrannical", "rank"],
//	}
type AhnlichClientTestSuite struct {
	db     *exec.Cmd
	client *AhnlichClient
}

// setupDatabase returns a new instance of the AhnlichClientTestSuite
func setupDatabase(host string, port int) (*AhnlichClientTestSuite, error) {
	// Start the ahnlich database server
	// ahnlichclientgo.SetLogLevel(ahnlichclientgo.LogLevelDebug)
	rootDir, err := utils.GetProjectRoot()
	if err != nil {
		return nil, fmt.Errorf("failed to get project root: %w", err)
	}

	tomlDir := filepath.Join(rootDir, "..", "..", "ahnlich", "Cargo.toml")
	tomlDir, err = filepath.Abs(tomlDir)
	if err != nil {
		return nil, fmt.Errorf("failed to get absolute path: %w", err)
	}

	cmd := exec.Command("cargo", "run", "--manifest-path", tomlDir, "--bin", "ahnlich-db", "run", "--port", fmt.Sprint(port))
	var outBuf, errBuf bytes.Buffer
	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	if err := cmd.Start(); err != nil {
		return nil, fmt.Errorf("failed to start ahnlich database: %w", err)
	}

	// Wait for the database to start up
	maxRetries := 5
	retryInterval := 1 * time.Second

	for i := 0; i < maxRetries; i++ {
		// check if the database is running
		if cmd.ProcessState != nil {
			if cmd.ProcessState.Exited() || cmd.ProcessState.Success() {
				return nil, fmt.Errorf("failed to start ahnlich database: %v", cmd.ProcessState)
			}
		}
		// Checking stderr for the Running message as well because the database writes warnings to stderr also
		if strings.Contains(outBuf.String(), "Running") || (strings.Contains(errBuf.String(), "Running") && strings.Contains(errBuf.String(), "Finished")) {
			break
		}
		if i == maxRetries-1 {
			cmd.Process.Kill()
			return nil, fmt.Errorf("database did not start within the expected time")
		}
		time.Sleep(retryInterval)
	}

	// Check for any errors in stderr
	if strings.Contains(errBuf.String(), "error:") {
		cmd.Process.Kill()
		return nil, fmt.Errorf("failed to start ahnlich database: %s", errBuf.String())
	}
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
	if err != nil {
		cmd.Process.Kill()
		return nil, err
	}
	dbClient, err := NewAhnlichClient(cm, config.ClientConfig)
	if err != nil {
		cmd.Process.Kill()
		return nil, err
	}

	return &AhnlichClientTestSuite{
		db:     cmd,
		client: dbClient,
	}, nil
}

// teardownDatabase stops the custom database
func (ts *AhnlichClientTestSuite) teardownDatabase() {
	// Stop the custom database server
	ts.client.Close()
	if ts.db != nil {
		ts.db.Process.Signal(os.Interrupt)
		ts.db.Wait()
	}
}

func TestNewAhnlichClient(t *testing.T) {
	testSuite, err := setupDatabase("localhost", 1100)
	require.NoError(t, err)
	defer testSuite.teardownDatabase()

	info, _ := testSuite.client.ServerInfo()

	fmt.Println(info)

	fmt.Println(testSuite.client.Version())

	fmt.Println(testSuite.client.ProtocolVersion())

	fmt.Println(testSuite.client.Ping())

}
