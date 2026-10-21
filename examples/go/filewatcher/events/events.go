package events

import "time"

type EventType string

const (
	FileAdded   EventType = "FILE_ADDED"
	FileUpdated EventType = "FILE_UPDATED"
	FileRemoved EventType = "FILE_REMOVED"
)

type FileEvent struct {
	Type      EventType `json:"type"`
	FileName  string    `json:"file_name"`
	Path      string    `json:"path"`
	Timestamp time.Time `json:"timestamp"`
}
