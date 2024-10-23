package utils

import (
	"log/slog"
)

type Logger struct {
	// Logger is the logger used by the client
	Logger slog.Logger
}

// NewLogger creates a new instance of Logger
func NewLogger(logger slog.Logger) *Logger {
	return &Logger{
		Logger: logger,
	}
}
