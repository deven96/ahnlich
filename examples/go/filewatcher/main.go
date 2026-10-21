package main

import (
	"go/ahnlich/api"
	"go/ahnlich/cmd"
	"go/ahnlich/internals/watcher"
	"go/ahnlich/logger"
	"log"
	"os"

	"go.uber.org/zap"
)

func main() {
	cfg, err := cmd.ParseFlags()
	if err != nil {
		log.Fatal(err)
	}

	logr := logger.New(cfg.Verbose)
	defer logr.Sync()

	if _, err := os.Stat(cfg.Dir); err != nil {
		logr.Fatal("Directory not found", zap.Error(err))
	}

	apiClient := api.New(cfg.APIURL)
	w := watcher.New(logr, apiClient)

	if err := w.Watch(cfg.Dir); err != nil {
		logr.Fatal("Watcher failed", zap.Error(err))
	}
}
