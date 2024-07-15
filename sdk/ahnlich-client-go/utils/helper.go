package utils

import (
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
