
# Filewatcher

A lightweight Golang CLI tool that monitors a directory for PDF file changes and sends normalized file events to an external API in real time.

Designed to support downstream workflows such as AI analysis, embedding pipelines, and event logging.

---

## Features

* Real-time directory monitoring on macOS
* PDF-only file filtering
* Detects file lifecycle events:

  * `FILE_ADDED`
  * `FILE_UPDATED`
  * `FILE_REMOVED`
* Sends normalized events to an HTTP API
* macOS write-event debouncing
* Clean, extensible architecture for future integrations
* Verbose logging mode for debugging

---

## Requirements

* Go **1.22+**
* macOS (uses `fsnotify`, compatible with FSEvents)
* Network access to the target API

---

## Installation

Clone the repository:

```bash
git clone https://github.com/deven96/ahnlich.git
cd examples/go/filewatcher
```

Install dependencies:

```bash
go mod tidy
```

Build the binary:

```bash
go build -o filewatcher
```

---

## Usage

```bash
filewatcher --dir /path/to/watch --api-url http://localhost:8080/events
```

### Flags

| Flag        | Required | Description                         |
| ----------- | -------- | ----------------------------------- |
| `--dir`     | Yes      | Directory to monitor                |
| `--api-url` | Yes      | API endpoint to receive file events |
| `--verbose` | No       | Enable verbose (debug) logging      |

### Example

```bash
filewatcher \
  --dir /Users/michael/Documents \
  --api-url http://localhost:8080/file-events \
  --verbose
```

---

## Event Payload

Each detected file event is sent as JSON:

```json
{
  "type": "FILE_ADDED",
  "file_name": "invoice.pdf",
  "path": "/Users/michael/Documents/invoice.pdf",
  "timestamp": "2025-01-01T12:00:00Z"
}
```

---

## Supported Events

| Event Type     | Description                |
| -------------- | -------------------------- |
| `FILE_ADDED`   | New PDF file detected      |
| `FILE_UPDATED` | Existing PDF file modified |
| `FILE_REMOVED` | PDF file deleted           |

---

## Logging

Example logs:

```text
[INFO] Watching directory: /Users/chijooke/Documents
[INFO] FILE_ADDED: invoice.pdf
[INFO] Posted event to API
```

With `--verbose` enabled, additional debug-level logs are shown.

---

## Error Handling

* Directory not found → Program exits with a descriptive error
* API failure → Logged; watcher continues running
* Permission issues → Logged as warnings
* Non-PDF files → Ignored

---

## Project Structure

```text
filewatcher/
├── cmd/            # CLI flag parsing
├── internal/
│   ├── watcher/    # File system monitoring logic
│   ├── events/     # Normalized event definitions
│   ├── api/        # HTTP client
│   └── logger/     # Logging setup
├── main.go
└── README.md
```

---

## Roadmap / Future Enhancements

* Background retry queue for failed API calls
* File hashing to validate updates
* Built-in AI analysis pipeline
* Database persistence
* Scheduled or batched processing
* Integration with embedding storage (e.g. Ahnlich)

---

## Contributing

Contributions are welcome.

1. Fork the repository
2. Create a feature branch
3. Commit your changes with clear messages
4. Open a pull request

Please keep pull requests focused and well-documented.

---

## License

MIT License
