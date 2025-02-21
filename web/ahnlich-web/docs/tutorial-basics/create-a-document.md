---
sidebar_position: 2
---

# Example Docker Compose

<!-- Documents are **groups of pages** connected through:

- a **sidebar**
- **previous/next navigation**
- **versioning**

## Create your first Doc -->

<!-- Create a Markdown file at `docs/hello.md`: -->

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

<!-- A new document is now available at [http://localhost:3000/docs/hello](http://localhost:3000/docs/hello).

## Configure the Sidebar

Docusaurus automatically **creates a sidebar** from the `docs` folder.

Add metadata to customize the sidebar label and position:

```md title="docs/hello.md" {1-4}
---
sidebar_label: "Hi!"
sidebar_position: 3
---

# Hello

This is my **first Docusaurus document**!
```

It is also possible to create your sidebar explicitly in `sidebars.js`:

```js title="sidebars.js"
export default {
  tutorialSidebar: [
    "intro",
    // highlight-next-line
    "hello",
    {
      type: "category",
      label: "Tutorial",
      items: ["Installation/create-a-document"],
    },
  ],
};
``` -->
