---
sidebar_position: 2
---

# Example Docker Compose

```
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