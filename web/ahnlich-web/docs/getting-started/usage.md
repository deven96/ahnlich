---
title: 🖥️ Using the CLI
sidebar_position: 20
---

import Tabs from '@theme/Tabs';
import TabItem from '@theme/TabItem';

# Using the CLI

`ahnlich-cli` is an interactive shell for talking to Ahnlich **without writing
any code**. It's the fastest way to explore your data, run admin tasks, and
sanity-check queries.

:::tip When to use what
- **Building an app?** Use a client SDK — see the [Quickstart](./quickstart).
- **Need to install or run the servers?** See [Installation](./installation).
- **Exploring or administering?** You're in the right place.
:::

## Open the CLI

Make sure a server is running first (see [Installation](./installation)), then
connect the CLI to it. The `--agent` flag picks which service you're talking to:

<Tabs groupId="ahnlich-agent" queryString>
<TabItem value="db" label="DB Engine" default>

Talk directly to the vector store — you provide raw vectors yourself.

```bash
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

</TabItem>
<TabItem value="ai" label="AI Proxy">

Talk to the AI proxy — send raw text/images and it embeds them for you.

```bash
ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370
```

</TabItem>
</Tabs>

## The command language

Ahnlich uses a **declarative, SQL-like command style**. Most commands follow
this shape:

```bash
<COMMAND> <ARGS> IN <STORE> [WHERE (<predicate>)]
```

Commands are case-insensitive and can be chained with `;`. Both agents share the
same grammar — the only difference is that the **DB engine** takes raw vectors
while the **AI proxy** takes raw text/images. Pick an agent below; the tabs stay
in sync with the connection command above.

<Tabs groupId="ahnlich-agent" queryString>
<TabItem value="db" label="DB Engine" default>

#### Create a store

```bash
CREATESTORE test_store DIMENSION 2 PREDICATES (author, country)
```

#### Insert data

```bash
SET (([1.0, 2.1], {name: Haks, category: dev}), ([3.1, 4.8], {name: Deven, category: dev})) IN test_store
```

#### Retrieve by key

```bash
GETKEY ([1.0, 2.0], [3.0, 4.0]) IN test_store
```

#### Search by similarity

```bash
GETSIMN 2 WITH [1.0, 2.0] USING cosinesimilarity IN test_store WHERE (category = dev)
```

#### Chain commands

```bash
GETKEY ([1.0, 2.0]) IN test_store; CREATEPREDINDEX (name, category) IN test_store
```

</TabItem>
<TabItem value="ai" label="AI Proxy">

#### Create a store

```bash
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2 PREDICATES (author, category)
```

#### Insert data

```bash
SET (([This is the life of Haks], {name: Haks, category: dev}), ([This is the life of Deven], {name: Deven, category: dev})) IN my_store PREPROCESSACTION nopreprocessing
```

#### Search by meaning

```bash
GETSIMN 2 WITH [life of deven] USING cosinesimilarity IN my_store WHERE (category = dev)
```

:::info
You pass **raw text** — the AI proxy embeds it automatically before comparing it
against the stored vectors.
:::

</TabItem>
</Tabs>

## Command reference

<Tabs groupId="ahnlich-agent" queryString>
<TabItem value="db" label="DB Engine" default>

| Command | Description |
| --- | --- |
| `PING` | Check if the server is responsive |
| `LISTCLIENTS` | List active connections |
| `LISTSTORES [SCHEMA <schema>]` | List stores in a schema (defaults to `public`) |
| `INFOSERVER` | Get server metadata / version |
| `CREATESTORE <name> DIMENSION <n> ...` | Create a vector store |
| `CREATEPREDINDEX (k1, k2) IN <store>` | Create a predicate (metadata) index |
| `SET (...) IN <store>` | Insert one or more vectors |
| `GETKEY (<vector>) IN <store>` | Retrieve entries by exact key |
| `GETSIMN <n> WITH [<vector>] USING <metric> IN <store> WHERE (<predicate>)` | Query nearest neighbors |
| `DROPSTORE <name> IF EXISTS` | Delete a store |

</TabItem>
<TabItem value="ai" label="AI Proxy">

| Command | Description |
| --- | --- |
| `PING` | Check if the server is responsive |
| `LISTSTORES` | List all stores |
| `INFOSERVER` | Get server metadata / version |
| `CREATESTORE <name> QUERYMODEL <m> INDEXMODEL <m> ...` | Create an AI store bound to embedding models |
| `CREATEPREDINDEX (k1, k2) IN <store>` | Create a predicate (metadata) index |
| `SET (...) IN <store> PREPROCESSACTION <action>` | Insert raw text/images (embedded automatically) |
| `GETSIMN <n> WITH [<raw input>] USING <metric> IN <store> WHERE (<predicate>)` | Query nearest neighbors by meaning |
| `DROPSTORE <name> IF EXISTS` | Delete a store |

</TabItem>
</Tabs>

Supported metrics for `GETSIMN`: `cosinesimilarity`, `euclideandistance`, and
`dotproductsimilarity`.
