package utils

import (
	"bytes"
	"fmt"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

// AhnlichDBTestSuite is a test suite for the AhnlichDB
type AhnlichDBTestSuite struct {
	Host string
	Port int
	*exec.Cmd
}

func RunAhnlichDatabase(t *testing.T) *AhnlichDBTestSuite {
	var cmd *exec.Cmd
	host := "127.0.0.1" // localhost
	port, err := GetAvailablePort(host)
	require.NoError(t, err)

	t.Cleanup(func() {
		if cmd != nil {
			// Send an interrupt signal to the process
			cmd.Process.Signal(os.Interrupt)
			cmd.Wait()
			cmd.Process.Kill() // Ensure the process is killed
		}
	})
	rootDir, err := GetProjectRoot()
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

	return &AhnlichDBTestSuite{
		Host: host,
		Port: port,
		Cmd:  cmd,
	}
}

// GetAvailablePort finds an available port and returns it.
func GetAvailablePort(host string) (int, error) {
	maxRetries := 10
	delay := 100 * time.Millisecond
	for i := 0; i < maxRetries; i++ {
		listener, err := net.Listen("tcp", "localhost:0")
		if err == nil {
			addr := listener.Addr().(*net.TCPAddr)
			listener.Close()
			time.Sleep(delay * 3) // Small delay before returning the port
			return addr.Port, nil
		}
		fmt.Printf("Attempt %d: Error finding free port: %v\n", i+1, err)
		time.Sleep(delay) // Small delay before retrying
	}
	return 0, fmt.Errorf("unable to find a free port after %d attempts", maxRetries)
}
