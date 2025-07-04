package ahnlichgotest

import (
	"bytes"
	"encoding/json"
	"fmt"
	"go/build"
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
	MaxRetries    = 50
	RetryInterval = 1 * time.Second
)


type AhnlichProcess struct {
	ServerAddr string
	Host       string
	Port       int
	StdOut     *bytes.Buffer
	StdErr     *bytes.Buffer
	*exec.Cmd
}

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

type AddrsFlag struct {
	ServerAddr any
	host       string
	port       int
	baseFlag
}

func (args *AddrsFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
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
	args.Flags = append(args.Flags, "--port", fmt.Sprint(port))
	return args.Flags, nil
}

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

type BinaryFlag struct {
	BinaryType string // db, ai
	baseFlag
}

func (args *BinaryFlag) parseArgs() ([]string, error) {
	args.Flags = make([]string, 0)
	if args.BinaryType != "" {
		validTypes := []string{"db", "ai"}
		if !contains(validTypes, args.BinaryType) {
			args.Flags = append(args.Flags, "db")
		} else {
			args.Flags = append(args.Flags, args.BinaryType)
		}
	} else {
		args.Flags = append(args.Flags, "db")
	}
	args.BinaryType = args.Flags[0]
	return args.Flags, nil
}


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

func execute(t *testing.T,execType string, binType string, args ...string) (*exec.Cmd, error) {
	lookPath := "cargo"
	rootDir, err := GetPackageRoot("github.com/deven96/ahnlich/sdk/ahnlich-client-go")
	if err != nil {
		return nil, err
	}
	tomlDir := filepath.Join(rootDir, "..", "..", "ahnlich", "Cargo.toml")
	t.Log("execute() args","tomlDir", tomlDir, "rootDir", rootDir, "execType", execType, "args", args)
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
		commands = append(commands, "--bin", binType, "run")
	}
	commands = append(commands, args...)
	return exec.Command(lookPath, commands...), nil
}

// RunAhnlich starts the Ahnlich process
func RunAhnlich(t *testing.T, args ...OptionalFlags) *AhnlichProcess {
	var (
		cmd *exec.Cmd
		host string
		port int
		err error
		argsList []string
		execType string
		binaryType string
		outBuf bytes.Buffer
		errBuf bytes.Buffer

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


	cmd, err = execute(t,execType,binaryType, argsList...)
	require.NoError(t, err)
	require.NotEmpty(t, cmd)


	cmd.Stdout = &outBuf
	cmd.Stderr = &errBuf

	err = cmd.Start()
	require.NoError(t, err)

	// Wait for the ahnlich to start up

	for i := 0; i < MaxRetries; i++ {
		// check if the ahnlich is running
		if cmd.ProcessState != nil {
			require.Truef(t, !cmd.ProcessState.Exited(), "ahnlich process exited", outBuf.String(), errBuf.String())
			require.Truef(t, !cmd.ProcessState.Success(), "ahnlich process exited with success status", outBuf.String(), errBuf.String())
		}
		// Checking stderr for the Running message as well because the ahnlich writes warnings to stderr also
		if strings.Contains(outBuf.String(), "Running") || (strings.Contains(errBuf.String(), "Running") && strings.Contains(errBuf.String(), "Finished")) && (!strings.Contains(errBuf.String(), "panicked") || !strings.Contains(outBuf.String(), "panicked")) {
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

// Check if proc is running
func (proc *AhnlichProcess) IsRunning() bool {
	if proc.Cmd != nil {
		return (proc.Cmd.ProcessState != nil && !proc.Cmd.ProcessState.Exited()) || proc.Cmd.ProcessState == nil
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


func GetPackageRoot(pkgName string) (string, error) {
	// Look up the package in the Go build context
	pkg, err := build.Import(pkgName, "", build.FindOnly)
	if err != nil {
		return "", err
	}

	// Return the absolute path to the package root
	return filepath.Abs(pkg.Dir)
}


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
