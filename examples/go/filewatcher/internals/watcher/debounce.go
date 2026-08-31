package watcher

import "time"

type Debouncer struct {
	lastEvent map[string]time.Time
	window    time.Duration
}

func NewDebouncer(window time.Duration) *Debouncer {
	return &Debouncer{
		lastEvent: make(map[string]time.Time),
		window:    window,
	}
}

func (d *Debouncer) ShouldProcess(path string) bool {
	now := time.Now()
	last, ok := d.lastEvent[path]
	if ok && now.Sub(last) < d.window {
		return false
	}
	d.lastEvent[path] = now
	return true
}
