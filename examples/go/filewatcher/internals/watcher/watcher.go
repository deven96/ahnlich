package watcher

import (
	"go/ahnlich/api"
	"go/ahnlich/events"
	"path/filepath"
	"strings"
	"time"

	"github.com/fsnotify/fsnotify"
	"go.uber.org/zap"
)

type Watcher struct {
	logger    *zap.Logger
	apiClient *api.Client
	debouncer *Debouncer
}

func New(logger *zap.Logger, apiClient *api.Client) *Watcher {
	return &Watcher{
		logger:    logger,
		apiClient: apiClient,
		debouncer: NewDebouncer(500 * time.Millisecond),
	}
}

func (w *Watcher) Watch(dir string) error {
	watcher, err := fsnotify.NewWatcher()
	if err != nil {
		return err
	}
	defer watcher.Close()

	if err := watcher.Add(dir); err != nil {
		return err
	}

	w.logger.Info("Watching directory", zap.String("dir", dir))

	for {
		select {
		case event := <-watcher.Events:
			w.handleEvent(event)
		case err := <-watcher.Errors:
			w.logger.Warn("Watcher error", zap.Error(err))
		}
	}
}

func (w *Watcher) handleEvent(event fsnotify.Event) {
	if !strings.HasSuffix(strings.ToLower(event.Name), ".pdf") {
		return
	}

	if !w.debouncer.ShouldProcess(event.Name) {
		return
	}

	var eventType events.EventType

	switch {
	case event.Op&fsnotify.Create == fsnotify.Create:
		eventType = events.FileAdded
	case event.Op&fsnotify.Write == fsnotify.Write:
		eventType = events.FileUpdated
	case event.Op&fsnotify.Remove == fsnotify.Remove:
		eventType = events.FileRemoved
	default:
		return
	}

	fileEvent := events.FileEvent{
		Type:      eventType,
		FileName:  filepath.Base(event.Name),
		Path:      event.Name,
		Timestamp: time.Now(),
	}

	w.logger.Info(string(eventType), zap.String("file", fileEvent.FileName))

	if err := w.apiClient.SendEvent(fileEvent); err != nil {
		w.logger.Warn("Failed to post event", zap.Error(err))
	}
}
