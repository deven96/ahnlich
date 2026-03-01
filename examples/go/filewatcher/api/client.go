package api

import (
	"bytes"
	"encoding/json"
	"go/ahnlich/events"
	"net/http"
	"time"
)

type Client struct {
	url    string
	client *http.Client
}

func New(apiURL string) *Client {
	return &Client{
		url: apiURL,
		client: &http.Client{
			Timeout: 10 * time.Second,
		},
	}
}

func (c *Client) SendEvent(event events.FileEvent) error {
	body, err := json.Marshal(event)
	if err != nil {
		return err
	}

	req, err := http.NewRequest(http.MethodPost, c.url, bytes.NewBuffer(body))
	if err != nil {
		return err
	}

	req.Header.Set("Content-Type", "application/json")
	_, err = c.client.Do(req)
	return err
}
