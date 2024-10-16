package utils

import (
	"errors"
	"fmt"
	"os"
	"path/filepath"
)

// GetProjectRoot returns the root directory of the project
func GetProjectRoot() (string, error) {
	// Get the current working directory
	cwd, err := os.Getwd()
	if err != nil {
		return "", fmt.Errorf("unable to get current working directory: %w", err)
	}
	projectRoot := filepath.Join(cwd, "..")
	// Clean the path to get the absolute path
	projectRoot, err = filepath.Abs(projectRoot)
	if err != nil {
		return "", fmt.Errorf("unable to resolve absolute path: %w", err)
	}
	return projectRoot, nil
}

// NonZeroUintStruct holds a uint that must be non-zero
type NonZeroUint struct {
	Value uint64
}

// NewNonZeroUint creates a new NonZeroUint ensuring the value is non-zero
func NewNonZeroUint(value uint64) (*NonZeroUint, error) {
	if value == 0 {
		return nil, errors.New("value cannot be zero")
	}
	return &NonZeroUint{Value: value}, nil
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
