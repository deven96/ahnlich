---
title: 📦 Installation
sidebar_position: 10
---

# Installation

## Docker

### Ahnlich AI

```
docker pull ghcr.io/deven96/ahnlich-ai:latest
```

### Ahnlich DB

```
docker pull ghcr.io/deven96/ahnlich-db:latest
```


## Docker compose


<details>
<summary><b>Docker Compose (Click to expand)</b></summary>

```docker
services:  
 ahnlich_db:  
 image: ghcr.io/deven96/ahnlich-db:latest  
 command: "ahnlich-db run --host 0.0.0.0 --enable-tracing"  
 ports:  
 - "1369:1369"

      ahnlich_ai:
        image: ghcr.io/deven96/ahnlich-ai:latest
        command: "ahnlich-ai run --db-host ahnlich_db --host 0.0.0.0 --enable-tracing --supported-models all-minilm-l6-v2,resnet-50"
        ports:
          - "1370:1370"
```

</details>

## Binary

### wget

```
wget https://github.com/deven96/ahnlich/releases/download/bin%2Fdb%2F0.0.0/aarch64-darwin-ahnlich-db.tar.gz
```

### extract the file

```
tar -xvzf aarch64-darwin-ahnlich-db.tar.gz
```

### run the library

```
./ahnlich-db 
```
