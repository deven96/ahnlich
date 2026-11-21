package cmd

import (
	"errors"
	"flag"
)

type Config struct {
	Dir     string
	APIURL  string
	Verbose bool
}

func ParseFlags() (*Config, error) {
	cfg := &Config{}

	flag.StringVar(&cfg.Dir, "dir", "", "Directory to watch")
	flag.StringVar(&cfg.APIURL, "api-url", "", "API endpoint")
	flag.BoolVar(&cfg.Verbose, "verbose", false, "Verbose logging")
	flag.Parse()

	if cfg.Dir == "" || cfg.APIURL == "" {
		return nil, errors.New("--dir and --api-url are required")
	}

	return cfg, nil
}
