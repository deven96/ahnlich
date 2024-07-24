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
// Remember the persistent interval when writing or running persistent tests
type AhnlichDBTestSuite struct {
	Host                string
	Port                int
	Persistence         bool
	PersistenceLocation string
	StdOut 			*bytes.Buffer
	StdErr 			*bytes.Buffer
	*exec.Cmd
}

func RunAhnlichDatabase(t *testing.T, persist bool, persistLocation string, serverAddr ...any) *AhnlichDBTestSuite {
	var cmd *exec.Cmd
	var host string
	var port int
	var portStr string
	var err error


	if len(serverAddr) == 2 || len(serverAddr) == 1 {
		for _, addr := range serverAddr {
			switch value := addr.(type) {
			case string:
				// check string format "host:port"
				host, portStr, err = net.SplitHostPort(value)
				if err != nil {
					host = value
				} else {
					port, err = net.LookupPort("tcp", portStr)
					require.NoError(t, err)
				}
			case int:
				port = value
			default:
				require.Fail(t, "invalid serverAddr format")
			}

		}
	} else {
		host = "127.0.0.1" // localhost
		port, err = GetAvailablePort(host)
		require.NoError(t, err)
	}

	require.NotEmpty(t, host)
	require.NotEmpty(t, port)

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
	if persist {
		cmd = exec.Command("cargo", "run", "--manifest-path", tomlDir, "--bin", "ahnlich-db", "run", "--port", fmt.Sprint(port),"--enable-persistence", "--persist-location", filepath.Join(persistLocation,"ahnlichdb.json"), "--persistence-interval", "10","--enable-tracing","--log-level","debug")
	} else {
		cmd = exec.Command("cargo", "run", "--manifest-path", tomlDir, "--bin", "ahnlich-db", "run", "--port", fmt.Sprint(port))
	}
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
			require.Truef(t, !cmd.ProcessState.Exited(), "database process exited",outBuf.String(),errBuf.String())
			require.Truef(t, !cmd.ProcessState.Success(), "database process exited with success status",outBuf.String(),errBuf.String())
		}
		// Checking stderr for the Running message as well because the database writes warnings to stderr also
		if strings.Contains(outBuf.String(), "Running") || (strings.Contains(errBuf.String(), "Running") && strings.Contains(errBuf.String(), "Finished")) && (!strings.Contains(errBuf.String(),"panicked") || !strings.Contains(outBuf.String(),"panicked")) {
			break
		}
		require.Truef(t, i < maxRetries-1, "database did not start within the expected time %v", retryInterval*time.Duration(maxRetries),outBuf,errBuf)
		time.Sleep(retryInterval)
	}

	// Check for any errors in stderr
	require.Truef(t, !strings.Contains(errBuf.String(), "error:"), "failed to start ahnlich database: %s", errBuf.String())
	// Check for any panicked in stderr or stdout
	require.Truef(t, !strings.Contains(errBuf.String(), "panicked"), "ahnlich database panicked: %s", errBuf.String())
	require.Truef(t, !strings.Contains(outBuf.String(), "panicked"), "ahnlich database panicked: %s", outBuf.String())

	return &AhnlichDBTestSuite{
		Host:                host,
		Port:                port,
		Persistence:         persist,
		PersistenceLocation: persistLocation,
		Cmd:                 cmd,
		StdOut: &outBuf,
		StdErr: &errBuf,
	}
}

// Kill stops the AhnlichDB process
func (db *AhnlichDBTestSuite) Kill() {
	if db.Cmd != nil {
		// Send an interrupt signal to the process
		db.Cmd.Process.Signal(os.Interrupt)
		db.Cmd.Wait()
		db.Cmd.Process.Kill()
	}

}

// Check if db is running
func (db *AhnlichDBTestSuite) IsRunning() bool {
	if db.Cmd != nil {
		return (db.Cmd.ProcessState != nil && !db.Cmd.ProcessState.Exited()) || db.Cmd.ProcessState == nil
	}
	return false
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