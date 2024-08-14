package utils

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io"
	"net"
	"os"
	"os/exec"
	"path/filepath"
	"reflect"
	"strconv"
	"strings"
	"testing"
	"time"

	"github.com/stretchr/testify/require"
)

// AhnlichDBTestSuite is a test suite for the AhnlichDB

type AhnlichDBTestSuite struct {
	ServerAddr string
	Host       string
	Port       int
	StdOut     *bytes.Buffer
	StdErr     *bytes.Buffer
	*exec.Cmd
}

type OptionalArgs interface {
	// parseArgs takes in a struct with the args and returns a slice of strings
	parseArgs() ([]string, error)
	getName(i interface{}) string
}

type baseOption struct {
	Args []string
}

func (args baseOption) getName(i interface{}) string {
	// Use reflection to get the type of the input
	t := reflect.TypeOf(i)

	// Ensure we're working with a pointer, and get the element type if so
	if t.Kind() == reflect.Ptr {
		t = t.Elem()
	}

	// Return the name of the struct
	return t.Name()
}

type AddrsOption struct {
	ServerAddr any
	host       string
	port       int
	baseOption
}

func (args *AddrsOption) parseArgs() ([]string, error) {
	args.Args = make([]string, 0)
	var host string = "127.0.0.1"
	var port int
	var portStr string
	var err error

	if args.ServerAddr != nil || args.ServerAddr != "" {
		switch value := args.ServerAddr.(type) {
		case string:
			// check string format "host:port"
			host, portStr, err = net.SplitHostPort(value)
			if err != nil {
				host = value
			} else {
				port, err = strconv.Atoi(portStr)
				if err != nil {
					return nil, err
				}
			}
		case int:
			port = value
		default:
			port, err = GetAvailablePort(host)
			if err != nil {
				return nil, err
			}
		}
	} else {
		port, err = GetAvailablePort(host)
		if err != nil {
			return nil, err
		}
	}
	args.host = host
	args.port = port
	args.Args = append(args.Args, "--port", fmt.Sprint(port))
	return args.Args, nil
}

type PersistOption struct {
	Persistence             bool
	PersistenceFileLocation string
	PersistenceInterval     int
	baseOption
}

func (args *PersistOption) parseArgs() ([]string, error) {
	args.Args = make([]string, 0)
	if args.Persistence {
		args.Args = append(args.Args, "--enable-persistence")
		if args.PersistenceFileLocation != "" {
			args.Args = append(args.Args, "--persist-location", args.PersistenceFileLocation)
		} else {
			args.Args = append(args.Args, "--persist-location", filepath.Join(".", "ahnlichdb.json"))
		}
		if args.PersistenceInterval != 0 {
			args.Args = append(args.Args, "--persistence-interval", fmt.Sprint(args.PersistenceInterval))
		} else {
			args.Args = append(args.Args, "--persistence-interval", "100")
		}
	}
	return args.Args, nil
}

type LogOption struct {
	LogLevel       string // trace, debug, info, warn, error
	TracingEnabled bool
	baseOption
}

func (args *LogOption) parseArgs() ([]string, error) {
	args.Args = make([]string, 0)
	if args.LogLevel != "" {
		logLevel := strings.ToLower(args.LogLevel)
		acceptedLogLevels := []string{"trace", "debug", "info", "warn", "error"}
		if !contains(acceptedLogLevels, logLevel) {
			logLevel = "info"
		}
		args.Args = append(args.Args, "--log-level", logLevel)
	} else {
		args.Args = append(args.Args, "--log-level", "info")
	}
	if args.TracingEnabled {
		args.Args = append(args.Args, "--enable-tracing")
	}
	return args.Args, nil
}

type ClientsOption struct {
	MaximumClients int
	baseOption
}

func (args *ClientsOption) parseArgs() ([]string, error) {
	args.Args = make([]string, 0)
	if args.MaximumClients != 0 {
		args.Args = append(args.Args, "--maximum-clients", fmt.Sprint(args.MaximumClients))
	} else {
		args.Args = append(args.Args, "--maximum-clients", "10")
	}
	return args.Args, nil
}

type ExecOption struct {
	ExecType string
	baseOption
}

func (args *ExecOption) parseArgs() ([]string, error) {
	args.Args = make([]string, 0)
	if args.ExecType != "" {
		validTypes := []string{"run", "build"}
		if !contains(validTypes, args.ExecType) {
			args.Args = append(args.Args, "run")
		} else {
			args.Args = append(args.Args, args.ExecType)
		}
	} else {
		args.Args = append(args.Args, "run")
	}
	args.ExecType = args.Args[0]
	return args.Args, nil
}

func execDb(execType string, args ...string) (*exec.Cmd, error) {
	lookPath := "cargo"
	rootDir, err := GetProjectRoot()
	if err != nil {
		return nil, err
	}
	tomlDir := filepath.Join(rootDir, "..", "..", "ahnlich", "Cargo.toml")
	tomlDir, err = filepath.Abs(tomlDir)
	if err != nil {
		return nil, err
	}
	if _, err := os.Stat(tomlDir); os.IsNotExist(err) {
		return nil, err
	}
	if _, err := exec.LookPath(lookPath); err != nil {
		return nil, err
	}
	commands := []string{execType, "--manifest-path", tomlDir}
	if execType == "run" {
		commands = append(commands, "--bin", "ahnlich-db", "run")
	}
	commands = append(commands, args...)
	return exec.Command(lookPath, commands...), nil
}

// RunAhnlichDatabase starts the AhnlichDB process
func RunAhnlichDatabase(t *testing.T, args ...OptionalArgs) *AhnlichDBTestSuite {
	var cmd *exec.Cmd
	var host string
	var port int
	var err error
	var argsList []string
	var execType string

	args = append(args, &ExecOption{}, &AddrsOption{})
	uniqueArgs := make(map[string]struct{})

	for _, opt := range args {

		if _, exists := uniqueArgs[opt.getName(opt)]; !exists {
			uniqueArgs[opt.getName(opt)] = struct{}{}
			switch arg := opt.(type) {
			case *ExecOption:
				_, err := opt.parseArgs()
				require.NoError(t, err)
				execType = arg.ExecType
				continue
			case *AddrsOption:
				parsedArgs, err := opt.parseArgs()
				require.NoError(t, err)
				host = arg.host
				port = arg.port
				argsList = append(argsList[:0], append(parsedArgs, argsList[0:]...)...) // Add the args to the beginning of the list
			default:
				parsedArgs, err := opt.parseArgs()
				require.NoError(t, err)
				argsList = append(argsList, parsedArgs...)
			}
		}

	}

	require.NotEmpty(t, host)
	require.NotEmpty(t, port)
	require.NotEmpty(t, execType)

	t.Cleanup(func() {
		if cmd != nil {
			// Send an interrupt signal to the process
			cmd.Process.Signal(os.Interrupt)
			cmd.Wait()
			cmd.Process.Kill() // Ensure the process is killed
		}
	})

	cmd, err = execDb(execType, argsList...)
	require.NoError(t, err)
	require.NotEmpty(t, cmd)

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
			require.Truef(t, !cmd.ProcessState.Exited(), "database process exited", outBuf.String(), errBuf.String())
			require.Truef(t, !cmd.ProcessState.Success(), "database process exited with success status", outBuf.String(), errBuf.String())
		}
		// Checking stderr for the Running message as well because the database writes warnings to stderr also
		if strings.Contains(outBuf.String(), "Running") || (strings.Contains(errBuf.String(), "Running") && strings.Contains(errBuf.String(), "Finished")) && (!strings.Contains(errBuf.String(), "panicked") || !strings.Contains(outBuf.String(), "panicked")) {
			break
		}
		require.Truef(t, i < maxRetries-1, "database did not start within the expected time %v", retryInterval*time.Duration(maxRetries), outBuf.String(), errBuf.String())
		time.Sleep(retryInterval)
	}

	// Check for any errors in stderr
	require.Truef(t, !strings.Contains(errBuf.String(), "error:"), "failed to start ahnlich database: %s", errBuf.String())
	// Check for any panicked in stderr or stdout
	require.Truef(t, !strings.Contains(errBuf.String(), "panicked"), "ahnlich database panicked: %s", errBuf.String())
	require.Truef(t, !strings.Contains(outBuf.String(), "panicked"), "ahnlich database panicked: %s", outBuf.String())

	return &AhnlichDBTestSuite{
		ServerAddr: fmt.Sprintf("%s:%d", host, port),
		Host:       host,
		Port:       port,
		StdOut:     &outBuf,
		StdErr:     &errBuf,
		Cmd:        cmd,
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

func ValidateJsonFile(t *testing.T, jsonFilePath string) {
	// Open the JSON file
	file, err := os.Open(jsonFilePath)
	require.NoError(t, err)
	defer file.Close()

	// Read the file content
	content, err := io.ReadAll(file)
	require.NoError(t, err)
	require.NotEmpty(t, content)

	// Optional: Unmarshal the JSON to validate its structure
	var data interface{}
	err = json.Unmarshal(content, &data)
	require.NoError(t, err)
}
