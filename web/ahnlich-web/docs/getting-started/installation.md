---
title: 📦 Installation
sidebar_position: 10
---



## 🐳 Install via **Docker** *(Recommended for isolated environments & CI)*

Pull the latest official container images:

```bash  
docker pull ghcr.io/deven96/ahnlich-db:latest  
docker pull ghcr.io/deven96/ahnlich-ai:latest  
```

Run both services locally with default ports (DB → 1369, AI → 1370):

```bash  
docker run -d --name ahnlich-db -p 1369:1369 ghcr.io/deven96/ahnlich-db:latest

docker run -d \
  --name ahnlich-ai \
  -p 1370:1370 \
  ghcr.io/deven96/ahnlich-ai:latest 
```

For more advanced setups—including tracing, persistence, and model caching—refer to the example [`docker-compose.yml`](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml) in the main repository.



## **Download Pre-built Binaries** *(Great for local servers & headless deployment)*

You can download OS‑specific binaries (for `db` and `ai`) from the [Ahnlich GitHub Releases page](https://github.com/deven96/ahnlich/releases). [GitHub](https://github.com/deven96/ahnlich/releases?utm_source=chatgpt.com)

Example steps for a Linux (`x86_64-unknown-linux-gnu`) environment:

```bash  
# Download the "db" binary for your version/platform
wget -L https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.1.0/x86_64-unknown-linux-gnu-ahnlich-db.tar.gz
Verify it’s a gzip file

Extract the contents
tar -xvzf ahnlich-db.tar.gz

Make the binary executable
chmod +x ahnlich-db

Run the database service
./ahnlich-db

./ahnlich-db --help
```

Repeat the same for the `ahnlich-ai` binary, substituting `db` → `ai` and the correct filename.

You can find complete download instructions (including Windows / macOS options) in the [official repository README](https://github.com/deven96/ahnlich/blob/main/README.md). [GitHub](https://github.com/deven96/ahnlich?utm_source=chatgpt.com)

---

## **Build from Source with Cargo** *(For developers and Rust contributors)*

Get up-to-date source and compile the binaries natively:

```bash  
git clone https://github.com/deven96/ahnlich.git
cd ahnlich
cargo build --release --bin ahnlich-db
cargo build --release --bin ahnlich-ai
```

Once built, find the executables in `target/release/`. Move them into your `$PATH` or launch directly:

```bash  
./target/release/ahnlich-db --help
./target/release/ahnlich-ai --help
```

This method is ideal for reviewing code, customizing defaults, or staying on the cutting edge. Ensure you have the Rust toolchain installed via [rustup.rs](https://rustup.rs/). [GitHub](https://github.com/deven96/ahnlich/blob/main/README.md?utm_source=chatgpt.com)



## **✅ Quick Comparison Table**

| Method | External Dependencies | Best For | Upgrade Workflow |
| ----- | ----- | ----- | ----- |
| **Docker** | Docker only | Isolated sandbox, CI, testing | `docker pull ghcr.io/deven96/ahnlich-*` |
| **Official binaries** | None (just download tool) | Servers without Rust or Docker | Download updated files from Releases |
| **Source (Cargo)** | Rust toolchain | Custom builds, contributions | `git pull && cargo build` |



## **🛠️ Post‑Installation Checklist**

* Add execution permissions: **`chmod +x <binary>`**.

* Ports: **ahnlich-db** defaults to `1369`, **ahnlich-ai** defaults to `1370`. Use `--host` and `--port` flags to customize.

* For upgrades:

  * **Docker**: pull the `:latest` tag;

  * **Binaries**: download again from the releases page;

  * **Source**: run `git pull && cargo build`.



## **🔗 Helpful Links**

* 🏠 [Main repository & documentation](https://github.com/deven96/ahnlich)

* 📦 [Releases page for downloading binaries](https://github.com/deven96/ahnlich/releases) [GitHub](https://github.com/deven96/ahnlich/releases?utm_source=chatgpt.com)

* 🧾 [Example Docker Compose and usage docs](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml) [GitHub](https://github.com/deven96/ahnlich/blob/main/docker-compose.yml?utm_source=chatgpt.com)

* 📖 [Full README (includes installation & usage guidance)](https://github.com/deven96/ahnlich/blob/main/README.md) [GitHub](https://github.com/deven96/ahnlich/blob/main/README.md?utm_source=chatgpt.com)  