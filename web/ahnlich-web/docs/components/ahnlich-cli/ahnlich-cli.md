---
title: 📟 Ahnlich CLI
sidebar_position: 10
---

# Ahnlich CLI

The **Ahnlich Command-Line Interface (CLI)** is a lightweight tool that allows developers, researchers, and operators to interact directly with the **Ahnlich AI** and **Ahnlich DB** servers.
It provides a simple way to issue commands using a custom **Domain-Specific Language (DSL)** without writing any code.

Think of the CLI as your **playground** for exploring Ahnlich:

- **Test queries quickly** without setting up an SDK project

- **Experiment with similarity search** and vector operations interactively

- **Prototype pipelines** for embedding, storage, and retrieval

- **Debug servers locally** before moving to production


Although the CLI is a powerful tool for testing, it is not intended as the main integration method. For production applications, the **Ahnlich SDKs (Rust, Go, Python)** should be used, as they provide richer APIs, better error handling, and integration capabilities.

---

## Non-Interactive Mode

The CLI supports a **non-interactive mode** via the `--no-interactive` flag, which is ideal for:

- **Docker health checks** - Verify server availability in container orchestration
- **CI/CD pipelines** - Automate testing and deployment workflows
- **Shell scripts** - Integrate Ahnlich commands into automation scripts
- **Monitoring systems** - Programmatically check server health and status

### Usage

In non-interactive mode, the CLI reads commands from **stdin** and exits immediately after processing:

```bash
# Single command via echo
echo 'PING' | ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369 --no-interactive

# Multiple commands via heredoc
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369 --no-interactive <<EOF
PING
INFOSERVER
LISTSTORES
EOF

# Commands from a file
cat commands.txt | ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370 --no-interactive
```

### Docker Health Check Example

```yaml
services:
  ahnlich_db:
    image: ghcr.io/deven96/ahnlich-db:latest
    command: "ahnlich-db run --host 0.0.0.0"
    ports:
      - "1369:1369"
    healthcheck:
      test: ["CMD-SHELL", "echo 'PING' | ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369 --no-interactive"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 5s

  ahnlich_ai:
    image: ghcr.io/deven96/ahnlich-ai:latest
    command: "ahnlich-ai run --db-host ahnlich_db --host 0.0.0.0"
    ports:
      - "1370:1370"
    depends_on:
      ahnlich_db:
        condition: service_healthy
    healthcheck:
      test: ["CMD-SHELL", "echo 'PING' | ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370 --no-interactive"]
      interval: 10s
      timeout: 5s
      retries: 3
      start_period: 10s
```

### Exit Codes

- **0**: All commands executed successfully
- **Non-zero**: Error occurred (connection failure, invalid command, etc.)

This makes it easy to integrate with shell scripts and monitoring tools that rely on exit codes for success/failure detection.
