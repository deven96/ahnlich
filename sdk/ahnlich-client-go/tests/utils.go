// Package ahnlichgotest ... provides utilities to run and manage the Ahnlich process in tests.
package ahnlichgotest

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

const (
	MaxRetries    = 120             // MaxRetries ... Maximum number of retries to check if the Ahnlich process is running
	RetryInterval = 1 * time.Second // RetryInterval ... Interval between retries to check if the Ahnlich process is running
)

// AhnlichProcess ... A struct to hold the Ahnlich process information
type AhnlichProcess struct {
	ServerAddr string
	Host       string
	Port       int
	StdOut     *bytes.Buffer
	StdErr     *bytes.Buffer
	*exec.Cmd
}

// OptionalFlags ... An interface to define the methods for optional flags
type OptionalFlags interface {
	// parseArgs takes in a struct with the args and returns a slice of strings
	parseArgs() ([]string, error)
	getName(i interface{}) string
}

type baseFlag struct {
	Flags []string
}

func (args baseFlag) getName(i interface{}) string {
	// Use reflection to get the type of the input
	t := reflect.TypeOf(i)

	// Ensure we're working with a pointer, and get the element type if so
	if t.Kind() == reflect.Ptr {
		t = t.Elem()
	}

	// Return the name of the struct
	return t.Name()
}

// AddrsFlag ... A struct to hold the server address and port
type AddrsFlag struct {
	ServerAddr any
	host       string
	port       int
	baseFlag
}

func (args *AddrsFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	var host = "127.0.0.1"
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
			port, err = GetAvailablePort()
			if err != nil {
				return nil, err
			}
		}
	} else {
		port, err = GetAvailablePort()
		if err != nil {
			return nil, err
		}
	}
	args.host = host
	args.port = port
	args.Flags = append(args.Flags, "--port", fmt.Sprint(port))
	return args.Flags, nil
}

// PersistFlag ... A struct to hold the persistence related flags
type PersistFlag struct {
	Persistence             bool
	PersistenceFileLocation string
	PersistenceInterval     int
	baseFlag
}

func (args *PersistFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.Persistence {
		args.Flags = append(args.Flags, "--enable-persistence")
		if args.PersistenceFileLocation != "" {
			args.Flags = append(args.Flags, "--persist-location", args.PersistenceFileLocation)
		} else {
			args.Flags = append(args.Flags, "--persist-location", filepath.Join(".", "ahnlich.json"))
		}
		if args.PersistenceInterval != 0 {
			args.Flags = append(args.Flags, "--persistence-interval", fmt.Sprint(args.PersistenceInterval))
		} else {
			args.Flags = append(args.Flags, "--persistence-interval", "100")
		}
	}
	return args.Flags, nil
}

// BinaryFlag ... A struct to hold the binary type (db or ai)
type BinaryFlag struct {
	BinaryType string // ahnlich-db or ahnlich-ai
	baseFlag
}

func (args *BinaryFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.BinaryType != "" {
		validTypes := []string{"ahnlich-db", "ahnlich-ai"}
		if !contains(validTypes, args.BinaryType) {
			args.Flags = append(args.Flags, "ahnlich-db")
		} else {
			args.Flags = append(args.Flags, args.BinaryType)
		}
	} else {
		args.Flags = append(args.Flags, "ahnlich-db")
	}
	args.BinaryType = args.Flags[0]
	return args.Flags, nil
}

// LogFlag ... A struct to hold the logging related flags
type LogFlag struct {
	LogLevel       string // trace, debug, info, warn, error
	TracingEnabled bool
	baseFlag
}

func (args *LogFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.LogLevel != "" {
		logLevel := strings.ToLower(args.LogLevel)
		acceptedLogLevels := []string{"trace", "debug", "info", "warn", "error"}
		if !contains(acceptedLogLevels, logLevel) {
			logLevel = "info"
		}
		args.Flags = append(args.Flags, "--log-level", logLevel)
	} else {
		args.Flags = append(args.Flags, "--log-level", "info")
	}
	if args.TracingEnabled {
		args.Flags = append(args.Flags, "--enable-tracing")
	}
	return args.Flags, nil
}

// ClientsFlag ... A struct to hold the maximum number of clients
type ClientsFlag struct {
	MaximumClients int
	baseFlag
}

func (args *ClientsFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.MaximumClients != 0 {
		args.Flags = append(args.Flags, "--maximum-clients", fmt.Sprint(args.MaximumClients))
	} else {
		args.Flags = append(args.Flags, "--maximum-clients", "10")
	}
	return args.Flags, nil
}

// ExecFlag ... A struct to hold the execution type (run or build)
type ExecFlag struct {
	ExecType string
	baseFlag
}

func (args *ExecFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.ExecType != "" {
		validTypes := []string{"run", "build"}
		if !contains(validTypes, args.ExecType) {
			args.Flags = append(args.Flags, "run")
		} else {
			args.Flags = append(args.Flags, args.ExecType)
		}
	} else {
		args.Flags = append(args.Flags, "run")
	}
	args.ExecType = args.Flags[0]
	return args.Flags, nil
}

func execute(t *testing.T, execType string, binType string, args ...string) (*exec.Cmd, error) {
	t.Log("Start execute() args", "execType", execType, "binType", binType, "args", args)
	rootDir, err := GetPackageRoot(t)
	if err != nil {
		return nil, err
	}
	t.Log("execute() args", "rootDir", rootDir, "execType", execType, "args", args)
	serverPath := filepath.Join(rootDir, "..", "..", "ahnlich")

	t.Log("execute() args", "rootDir", rootDir, "execType", execType, "args", args)
	lookPath := "cargo"
	commands := []string{execType}
	if execType == "run" {
		commands = append(commands, "--bin", binType, "run")
	}
	if _, err := exec.LookPath(lookPath); err != nil {
		return nil, err
	}
	commands = append(commands, args...)
	cmd := exec.Command(lookPath, commands...)
	cmd.Dir = serverPath
	return cmd, nil
}

// RunAhnlich starts the Ahnlich process
func RunAhnlich(t *testing.T, args ...OptionalFlags) *AhnlichProcess {
	var (
		cmd        *exec.Cmd
		host       string
		port       int
		err        error
		argsList   []string
		execType   string
		binaryType string
		outBuf     bytes.Buffer
		errBuf     bytes.Buffer
	)
	t.Cleanup(func() {
		if cmd != nil {
			// Send an interrupt signal to the process
			cmd.Process.Signal(os.Interrupt)
			cmd.Wait()
			cmd.Process.Kill() // Ensure the process is killed
			t.Log("RunAhnlich() cleanup: ahnlich stopped", "host", host, "port", port, "execType", execType, "args", argsList, "stdout", outBuf.String(), "stderr", errBuf.String(), "cmd", cmd)
		}
	})

	args = append(args, &ExecFlag{}, &AddrsFlag{})
	uniqueArgs := make(map[string]struct{})

	for _, opt := range args {

		if _, exists := uniqueArgs[opt.getName(opt)]; !exists {
			uniqueArgs[opt.getName(opt)] = struct{}{}
			switch arg := opt.(type) {
			case *ExecFlag:
				_, err := opt.parseArgs()
				require.NoError(t, err)
				execType = arg.ExecType
			case *AddrsFlag:
				parsedArgs, err := opt.parseArgs()
				require.NoError(t, err)
				host = arg.host
				port = arg.port
				argsList = append(argsList[:0], append(parsedArgs, argsList[0:]...)...) // Add the args to the beginning of the list
			case *BinaryFlag:
				_, err := opt.parseArgs()
				require.NoError(t, err)
				binaryType = arg.BinaryType
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
	require.NotEmpty(t, argsList)
	t.Log("RunAhnlich() args", "host", host, "port", port, "execType", execType, "args", argsList)
	var dbAhlichProcess *AhnlichProcess
	if binaryType == "ahnlich-ai" {
		dbAhlichProcess = RunAhnlich(t, &BinaryFlag{BinaryType: "ahnlich-db"})
	}
	if dbAhlichProcess != nil {
		argsList = append(argsList, "--db-host", dbAhlichProcess.Host, "--db-port", fmt.Sprint(dbAhlichProcess.Port))
	}
	cmd, err = execute(t, execType, binaryType, argsList...)
	require.NoError(t, err)
	require.NotEmpty(t, cmd)

	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	t.Log("RunAhnlich() cmd", "cmd", cmd)
	err = cmd.Start()
	require.NoError(t, err)

	// Wait for the ahnlich to start up

	for i := 0; i < MaxRetries; i++ {
		// check if the ahnlich is running
		if cmd.ProcessState != nil {
			require.Truef(t, !cmd.ProcessState.Exited(), "ahnlich process exited", outBuf.String(), errBuf.String())
			require.Truef(t, !cmd.ProcessState.Success(), "ahnlich process exited with success status", outBuf.String(), errBuf.String())
		}
		outBufString, errBufString := outBuf.String(), errBuf.String()

		// Checking stderr for the Running message as well because the ahnlich writes warnings to stderr also
		if strings.Contains(outBufString, "Running") || (strings.Contains(errBufString, "Running") && strings.Contains(errBufString, "Finished")) && (!strings.Contains(errBufString, "panicked") || !strings.Contains(outBufString, "panicked")) {
			break
		}
		if (strings.Contains(outBufString, "Starting") || (strings.Contains(errBufString, "Starting"))) && (!strings.Contains(errBufString, "panicked") || !strings.Contains(outBufString, "panicked")) {
			break
		}
		t.Log("Waiting for the ahnlich to start")
		require.Truef(t, i < MaxRetries-1, "ahnlich did not start within the expected time %v", RetryInterval*time.Duration(MaxRetries), outBuf.String(), errBuf.String())
		time.Sleep(RetryInterval)
	}

	// Check for any errors in stderr
	require.Truef(t, !strings.Contains(errBuf.String(), "error:"), "failed to start ahnlich ahnlich: %s", errBuf.String())
	// Check for any panicked in stderr or stdout
	require.Truef(t, !strings.Contains(errBuf.String(), "panicked"), "ahnlich ahnlich panicked: %s", errBuf.String())
	require.Truef(t, !strings.Contains(outBuf.String(), "panicked"), "ahnlich ahnlich panicked: %s", outBuf.String())

	t.Log("RunAhnlich() ahnlich started successfully", "host", host, "port", port, "execType", execType, "args", argsList, "stdout", outBuf.String(), "stderr", errBuf.String(), "cmd", cmd)
	return &AhnlichProcess{
		ServerAddr: fmt.Sprintf("%s:%d", host, port),
		Host:       host,
		Port:       port,
		StdOut:     &outBuf,
		StdErr:     &errBuf,
		Cmd:        cmd,
	}
}

// Kill stops the Ahnlich process
func (proc *AhnlichProcess) Kill() {
	if proc.Cmd != nil {
		// Send an interrupt signal to the process
		proc.Cmd.Process.Signal(os.Interrupt)
		proc.Cmd.Wait()
		proc.Cmd.Process.Kill()
	}

}

// IsRunning ... checks if the Ahnlich process is running
func (proc *AhnlichProcess) IsRunning() bool {
	if proc.Cmd != nil {
		return (proc.Cmd.ProcessState != nil && !proc.Cmd.ProcessState.Exited()) || proc.Cmd.ProcessState == nil
	}
	return false
}

// GetAvailablePort finds an available port and returns it.
func GetAvailablePort() (int, error) {
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

// ValidateJSONFile ... checks if the given JSON file is valid and not empty.
func ValidateJSONFile(t *testing.T, jsonFilePath string) {
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

// GetPackageRoot ... returns the root directory of the Go module.
func GetPackageRoot(t *testing.T) (string, error) {
	out, err := exec.Command("go", "env", "GOMOD").Output()
	if err != nil {
		return "", err
	}
	modPath := strings.TrimSpace(string(out))
	paths := strings.Split(modPath, string(os.PathSeparator))
	newPath := strings.Join(paths[:len(paths)-1], string(os.PathSeparator))
	if newPath == "" {
		return "", fmt.Errorf("not in a Go module")
	}
	t.Log("GetPackageRoot() module path", "modulePath", newPath)
	return newPath, nil
}

// ListFilesInDir ... lists all files in the given directory.
func ListFilesInDir(dir string) ([]string, error) {
	files, err := os.ReadDir(dir)
	if err != nil {
		return nil, fmt.Errorf("unable to read directory: %w", err)
	}
	var fileNames []string
	for _, file := range files {
		fileNames = append(fileNames, file.Name())
	}
	return fileNames, nil
}

// GetFileFromPath ... extracts the file name from a given path.
func GetFileFromPath(path string) string {
	file := filepath.Base(path)
	return file
}

func contains(slice []string, item string) bool {
	for _, element := range slice {
		if element == item {
			return true
		}
	}
	return false
}
