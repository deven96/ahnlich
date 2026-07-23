# Using ahnlich-wasm-db in Docusaurus

## Installation

```bash
npm install @ahnlich/wasm-db
```

## Setup

### 1. Configure Headers

Docusaurus needs to serve with specific headers for SharedArrayBuffer support. Add to `docusaurus.config.js`:

```js
module.exports = {
  // ... other config
  
  plugins: [
    // Add custom headers plugin
    function customHeadersPlugin() {
      return {
        name: 'custom-headers-plugin',
        configureWebpack() {
          return {
            devServer: {
              headers: {
                'Cross-Origin-Opener-Policy': 'same-origin',
                'Cross-Origin-Embedder-Policy': 'require-corp',
              },
            },
          };
        },
      };
    },
  ],
};
```

### 2. Create a React Component

```tsx
// src/components/AhnlichDemo.tsx
import React, { useEffect, useState } from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';

function AhnlichDemoInner() {
  const [db, setDb] = useState(null);
  const [status, setStatus] = useState('Loading...');
  
  useEffect(() => {
    async function initDB() {
      try {
        // Dynamic import to avoid SSR issues
        const { default: init, AhnlichDB, initThreadPool } = await import('@ahnlich/wasm-db');
        
        // Initialize WASM
        await init();
        
        // Initialize thread pool (use 4 threads or navigator.hardwareConcurrency)
        setStatus('Initializing thread pool...');
        await initThreadPool(4);
        
        // Create DB instance
        const dbInstance = new AhnlichDB();
        setDb(dbInstance);
        setStatus('Ready!');
      } catch (error) {
        setStatus(`Error: ${error.message}`);
        console.error(error);
      }
    }
    
    initDB();
  }, []);
  
  return (
    <div>
      <h3>Ahnlich WASM-DB Status: {status}</h3>
      {db && <p>Database is ready! You can now call db.create_store(), etc.</p>}
    </div>
  );
}

export default function AhnlichDemo() {
  return (
    <BrowserOnly fallback={<div>Loading...</div>}>
      {() => <AhnlichDemoInner />}
    </BrowserOnly>
  );
}
```

### 3. Use in MDX

```mdx
---
title: Ahnlich Demo
---

import AhnlichDemo from '@site/src/components/AhnlichDemo';

# Try Ahnlich in Your Browser

<AhnlichDemo />
```

## API Usage

```typescript
import { AhnlichDB, CreateStore, StoreKey, StoreValue } from '@ahnlich/wasm-db';
import { CreateStore, Set, GetSimN } from '@ahnlich/wasm-db/protobuf-bundle';

// Create store
const createReq = new CreateStore({
  store: 'my-vectors',
  dimension: 128,
  createPredicates: ['category'],
  nonLinearIndices: [],
  errorIfExists: false,
  schema: 'default'
});

const result = db.create_store(createReq.toBinary());

// Insert vectors
const setReq = new Set({
  store: 'my-vectors',
  schema: 'default',
  entries: [/* ... */]
});

db.set(setReq.toBinary());

// Similarity search
const searchReq = new GetSimN({
  store: 'my-vectors',
  schema: 'default',
  searchInput: /* ... */,
  closest: 10
});

const results = db.get_sim_n(searchReq.toBinary());
```

## Browser Compatibility

Requires browsers with:
- WebAssembly threads support
- SharedArrayBuffer support
- Cross-origin isolation headers

Supported browsers:
- Chrome/Edge 91+
- Firefox 89+
- Safari 15.2+

## Troubleshooting

### "SharedArrayBuffer is not defined"

Make sure you've configured the COOP/COEP headers correctly. For production deployment (not just Docusaurus dev server), you'll need to configure your hosting provider.

**Vercel:**
```json
// vercel.json
{
  "headers": [
    {
      "source": "/(.*)",
      "headers": [
        { "key": "Cross-Origin-Opener-Policy", "value": "same-origin" },
        { "key": "Cross-Origin-Embedder-Policy", "value": "require-corp" }
      ]
    }
  ]
}
```

**Netlify:**
```toml
# netlify.toml
[[headers]]
  for = "/*"
  [headers.values]
    Cross-Origin-Opener-Policy = "same-origin"
    Cross-Origin-Embedder-Policy = "require-corp"
```
