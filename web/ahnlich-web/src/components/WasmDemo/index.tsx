import React, { useEffect, useState, useRef } from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';

interface VectorItem {
  id: string;
  label: string;
  vector: number[];
  category: string;
  x: number; // 2D projection
  y: number;
}

interface SearchResult {
  item: VectorItem;
  similarity: number;
}

// Expanded product catalog with emojis and more variety
const SAMPLE_PRODUCTS = [
  // Electronics
  { label: '💻 Laptop', category: 'Electronics', baseVector: [0.85, 0.1, 0.15, 0.2] },
  { label: '🖱️ Mouse', category: 'Electronics', baseVector: [0.75, 0.15, 0.2, 0.25] },
  { label: '⌨️ Keyboard', category: 'Electronics', baseVector: [0.78, 0.12, 0.18, 0.22] },
  { label: '🖥️ Monitor', category: 'Electronics', baseVector: [0.82, 0.11, 0.16, 0.21] },
  { label: '🎧 Headphones', category: 'Electronics', baseVector: [0.7, 0.2, 0.25, 0.3] },
  { label: '📱 Smartphone', category: 'Electronics', baseVector: [0.88, 0.08, 0.12, 0.18] },
  { label: '⌚ Smartwatch', category: 'Electronics', baseVector: [0.8, 0.12, 0.2, 0.22] },
  { label: '📷 Camera', category: 'Electronics', baseVector: [0.77, 0.15, 0.22, 0.28] },
  { label: '🔌 Charger', category: 'Electronics', baseVector: [0.65, 0.2, 0.3, 0.25] },
  { label: '💾 USB Drive', category: 'Electronics', baseVector: [0.72, 0.18, 0.25, 0.28] },
  
  // Books & Media
  { label: '📚 Novel', category: 'Books', baseVector: [0.15, 0.85, 0.1, 0.2] },
  { label: '📖 Textbook', category: 'Books', baseVector: [0.2, 0.8, 0.15, 0.18] },
  { label: '📰 Magazine', category: 'Books', baseVector: [0.25, 0.75, 0.2, 0.22] },
  { label: '🗞️ Newspaper', category: 'Books', baseVector: [0.22, 0.78, 0.18, 0.2] },
  { label: '📕 Dictionary', category: 'Books', baseVector: [0.18, 0.82, 0.12, 0.16] },
  { label: '📓 Notebook', category: 'Books', baseVector: [0.28, 0.7, 0.25, 0.3] },
  { label: '✏️ Pen', category: 'Books', baseVector: [0.3, 0.68, 0.28, 0.32] },
  { label: '📝 Journal', category: 'Books', baseVector: [0.26, 0.72, 0.24, 0.28] },
  
  // Food & Beverages
  { label: '☕ Coffee', category: 'Food', baseVector: [0.1, 0.15, 0.85, 0.2] },
  { label: '🍵 Tea', category: 'Food', baseVector: [0.12, 0.18, 0.82, 0.22] },
  { label: '💧 Water', category: 'Food', baseVector: [0.08, 0.12, 0.78, 0.18] },
  { label: '🥤 Soda', category: 'Food', baseVector: [0.15, 0.2, 0.8, 0.25] },
  { label: '🍎 Apple', category: 'Food', baseVector: [0.2, 0.25, 0.75, 0.3] },
  { label: '🍌 Banana', category: 'Food', baseVector: [0.22, 0.28, 0.73, 0.32] },
  { label: '🥪 Sandwich', category: 'Food', baseVector: [0.25, 0.3, 0.7, 0.35] },
  { label: '🍕 Pizza', category: 'Food', baseVector: [0.28, 0.32, 0.68, 0.38] },
  { label: '🍔 Burger', category: 'Food', baseVector: [0.3, 0.35, 0.65, 0.4] },
  { label: '🥗 Salad', category: 'Food', baseVector: [0.18, 0.22, 0.77, 0.28] },
  
  // Clothing
  { label: '👕 T-Shirt', category: 'Clothing', baseVector: [0.2, 0.1, 0.15, 0.85] },
  { label: '👖 Jeans', category: 'Clothing', baseVector: [0.22, 0.12, 0.18, 0.82] },
  { label: '👟 Sneakers', category: 'Clothing', baseVector: [0.25, 0.15, 0.2, 0.8] },
  { label: '🧥 Jacket', category: 'Clothing', baseVector: [0.18, 0.08, 0.12, 0.88] },
  { label: '👗 Dress', category: 'Clothing', baseVector: [0.15, 0.1, 0.14, 0.87] },
  { label: '🧢 Cap', category: 'Clothing', baseVector: [0.28, 0.18, 0.22, 0.75] },
  { label: '🧣 Scarf', category: 'Clothing', baseVector: [0.2, 0.12, 0.16, 0.83] },
  { label: '👞 Shoes', category: 'Clothing', baseVector: [0.24, 0.14, 0.19, 0.81] },
  
  // Sports & Fitness
  { label: '⚽ Soccer Ball', category: 'Sports', baseVector: [0.3, 0.2, 0.1, 0.15] },
  { label: '🏀 Basketball', category: 'Sports', baseVector: [0.32, 0.22, 0.12, 0.18] },
  { label: '🎾 Tennis Ball', category: 'Sports', baseVector: [0.28, 0.18, 0.08, 0.12] },
  { label: '🏋️ Dumbbell', category: 'Sports', baseVector: [0.35, 0.25, 0.15, 0.2] },
  { label: '🧘 Yoga Mat', category: 'Sports', baseVector: [0.25, 0.15, 0.18, 0.22] },
  { label: '🎯 Dart', category: 'Sports', baseVector: [0.33, 0.23, 0.13, 0.17] },
  { label: '🏓 Ping Pong', category: 'Sports', baseVector: [0.29, 0.19, 0.09, 0.14] },
  { label: '🥊 Boxing Gloves', category: 'Sports', baseVector: [0.38, 0.28, 0.18, 0.25] },
  
  // Home & Garden
  { label: '🪴 Plant', category: 'Home', baseVector: [0.12, 0.25, 0.3, 0.08] },
  { label: '🕯️ Candle', category: 'Home', baseVector: [0.15, 0.28, 0.32, 0.1] },
  { label: '🛋️ Couch', category: 'Home', baseVector: [0.1, 0.22, 0.28, 0.06] },
  { label: '🪑 Chair', category: 'Home', baseVector: [0.11, 0.23, 0.29, 0.07] },
  { label: '🖼️ Frame', category: 'Home', baseVector: [0.14, 0.27, 0.31, 0.09] },
  { label: '💡 Lamp', category: 'Home', baseVector: [0.16, 0.26, 0.33, 0.11] },
  { label: '🧺 Basket', category: 'Home', baseVector: [0.13, 0.24, 0.3, 0.08] },
  { label: '🧹 Broom', category: 'Home', baseVector: [0.17, 0.29, 0.34, 0.12] },
];

function WasmDemoInner() {
  const [status, setStatus] = useState<string>('Click "Initialize Database" to start');
  const [db, setDb] = useState<any>(null);
  const [items, setItems] = useState<VectorItem[]>([]);
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [selectedItem, setSelectedItem] = useState<VectorItem | null>(null);
  const [isInitializing, setIsInitializing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [searchTime, setSearchTime] = useState<number>(0);
  const canvasRef = useRef<HTMLCanvasElement>(null);

  // Generate a high-dimensional vector from base 4D vector
  function generateVector(base: number[]): number[] {
    const dim = 128;
    const vec = new Array(dim);
    for (let i = 0; i < dim; i++) {
      const baseIdx = i % 4;
      vec[i] = base[baseIdx] + (Math.random() - 0.5) * 0.08;
    }
    return vec;
  }

  // 2D projection using different slices to create better separation
  function project2D(vector: number[]): { x: number; y: number } {
    // Use different patterns to avoid diagonal clustering
    // X: sum of even indices, Y: sum of odd indices
    let x = 0, y = 0;
    for (let i = 0; i < vector.length; i++) {
      if (i % 2 === 0) {
        x += vector[i];
      } else {
        y += vector[i];
      }
    }
    // Add some spread based on first few dimensions
    x += vector[0] * 0.3 + vector[2] * 0.2;
    y += vector[1] * 0.3 + vector[3] * 0.2;
    
    return { x, y };
  }

  async function initializeWasm() {
    setIsInitializing(true);
    setError(null);
    setStatus('Loading WASM module...');
    
    try {
      // Dynamic import from static files
      const wasmModule = await import('/wasm-pkg/ahnlich_wasm_db.js');
      const { default: init, AhnlichDB, initThreadPool } = wasmModule;
      
      // Initialize WASM
      await init();
      setStatus('Initializing multi-threading...');
      
      // Initialize thread pool
      const numThreads = Math.min(navigator.hardwareConcurrency || 4, 4);
      try {
        await initThreadPool(numThreads);
        setStatus(`Ready with ${numThreads} threads`);
      } catch (threadErr) {
        throw new Error(`Thread pool initialization failed. Make sure COOP/COEP headers are configured.`);
      }
      
      // Create DB instance
      const dbInstance = new AhnlichDB();
      
      // Load protobuf types
      const protobufModule = await import('/wasm-pkg/protobuf-bundle.js');
      const protobuf = protobufModule.default;
      const { queryPb, serverPb } = protobuf;
      
      // Create vector store
      setStatus('Creating vector store...');
      const createReq = new queryPb.CreateStore({
        store: 'products',
        dimension: 128,
        createPredicates: ['category'],
        nonLinearIndices: [],
        errorIfExists: false,
      });
      dbInstance.create_store(createReq.toBinary());
      
      // Create sample items
      setStatus('Adding sample products...');
      const sampleItems: VectorItem[] = SAMPLE_PRODUCTS.map((product, idx) => {
        const vector = generateVector(product.baseVector);
        const { x, y } = project2D(vector);
        return {
          id: `item-${idx}`,
          label: product.label,
          vector,
          category: product.category,
          x,
          y,
        };
      });
      
      // Insert into database
      const StoreKey = protobuf.StoreKey;
      const StoreValue = protobuf.StoreValue;
      const DbStoreEntry = protobuf.DbStoreEntry;
      const MetadataValue = protobuf.MetadataValue;
      
      const setReq = new queryPb.Set({
        store: 'products',
        inputs: sampleItems.map(item => new DbStoreEntry({
          key: new StoreKey({ key: item.vector }),
          value: new StoreValue({
            value: {
              id: new MetadataValue({ value: { case: 'rawString', value: item.id } }),
              label: new MetadataValue({ value: { case: 'rawString', value: item.label } }),
              category: new MetadataValue({ value: { case: 'rawString', value: item.category } }),
            }
          })
        }))
      });
      dbInstance.set(setReq.toBinary());
      
      setDb(dbInstance);
      setItems(sampleItems);
      setStatus('Ready! Click any item to find similar products.');
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('Failed to initialize');
    } finally {
      setIsInitializing(false);
    }
  }

  async function searchSimilar(item: VectorItem) {
    if (!db) return;
    
    const startTime = performance.now();
    
    // Clear previous results immediately
    setSearchResults([]);
    setSelectedItem(item);
    setStatus(`Searching for items similar to "${item.label}"...`);
    setError(null);
    
    try {
      const protobufModule = await import('/wasm-pkg/protobuf-bundle.js');
      const protobuf = protobufModule.default;
      const { queryPb, serverPb } = protobuf;
      const StoreKey = protobuf.StoreKey;
      
      const searchReq = new queryPb.GetSimN({
        store: 'products',
        searchInput: new StoreKey({ key: item.vector }),
        closestN: 6,
        algorithm: 2 // CosineSimilarity (0=Euclidean, 1=DotProduct, 2=Cosine)
      });
      
      const resultBytes = db.get_sim_n(searchReq.toBinary());
      const response = serverPb.GetSimN.fromBinary(new Uint8Array(resultBytes));
      
      // Extract results and match with our items
      const results: SearchResult[] = [];
      if (response.entries && response.entries.length > 0) {
        for (const entry of response.entries) {
          const matchedItem = items.find(i => {
            if (!entry.key?.key) return false;
            const entryVec = Array.from(entry.key.key);
            return i.vector.every((v, idx) => Math.abs(v - entryVec[idx]) < 0.0001);
          });
          
          if (matchedItem && matchedItem.id !== item.id) {
            const simValue = entry.similarity?.value ?? 0;
            results.push({ item: matchedItem, similarity: simValue });
          }
        }
      }
      
      const endTime = performance.now();
      setSearchTime(endTime - startTime);
      setSearchResults(results.slice(0, 5));
      setStatus(`Found ${results.length} similar items in ${(endTime - startTime).toFixed(2)}ms`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('Search failed');
    }
  }
  
  // Search by text query (fuzzy match on labels and find similar)
  function handleSearchQuery(query: string) {
    setSearchQuery(query);
    if (!query.trim() || !db || items.length === 0) {
      setSearchResults([]);
      setSelectedItem(null);
      setStatus('Ready! Type to search or click any item.');
      return;
    }
    
    // Find items that match the query
    const q = query.toLowerCase();
    const matches = items.filter(item => 
      item.label.toLowerCase().includes(q) || 
      item.category.toLowerCase().includes(q)
    );
    
    if (matches.length > 0) {
      // Search for items similar to the first match
      searchSimilar(matches[0]);
    } else {
      setSearchResults([]);
      setSelectedItem(null);
      setStatus('No matches found. Try "laptop", "book", "coffee", etc.');
    }
  }

  // Draw visualization canvas
  useEffect(() => {
    if (!canvasRef.current || items.length === 0) return;
    
    const canvas = canvasRef.current;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    const width = canvas.width;
    const height = canvas.height;
    
    // Clear canvas completely
    ctx.clearRect(0, 0, width, height);
    ctx.fillStyle = getComputedStyle(canvas).getPropertyValue('background-color') || '#ffffff';
    ctx.fillRect(0, 0, width, height);
    
    // Normalize coordinates
    const padding = 40;
    const xs = items.map(i => i.x);
    const ys = items.map(i => i.y);
    const minX = Math.min(...xs);
    const maxX = Math.max(...xs);
    const minY = Math.min(...ys);
    const maxY = Math.max(...ys);
    
    const scaleX = (x: number) => padding + ((x - minX) / (maxX - minX)) * (width - 2 * padding);
    const scaleY = (y: number) => padding + ((y - minY) / (maxY - minY)) * (height - 2 * padding);
    
    // Draw connections to search results with gradient
    if (selectedItem && searchResults.length > 0) {
      const sx = scaleX(selectedItem.x);
      const sy = scaleY(selectedItem.y);
      
      searchResults.forEach(({ item, similarity }) => {
        const ex = scaleX(item.x);
        const ey = scaleY(item.y);
        
        // Line thickness based on similarity (0.0 to 1.0) -> 1px to 8px
        const thickness = 1 + (similarity * 7);
        
        // Create gradient from selected to result
        const gradient = ctx.createLinearGradient(sx, sy, ex, ey);
        gradient.addColorStop(0, '#cc9200aa');
        gradient.addColorStop(1, `#cc920066`);
        
        ctx.strokeStyle = gradient;
        ctx.lineWidth = thickness;
        ctx.lineCap = 'round';
        ctx.beginPath();
        ctx.moveTo(sx, sy);
        ctx.lineTo(ex, ey);
        ctx.stroke();
        
        // Add similarity percentage label on the line
        const midX = (sx + ex) / 2;
        const midY = (sy + ey) / 2;
        const percent = Math.round(similarity * 100);
        
        ctx.fillStyle = '#cc9200';
        ctx.font = 'bold 12px system-ui';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(`${percent}%`, midX, midY - 10);
      });
    }
    
    // Draw items
    items.forEach(item => {
      const x = scaleX(item.x);
      const y = scaleY(item.y);
      
      const isSelected = selectedItem?.id === item.id;
      const isResult = searchResults.some(r => r.item.id === item.id);
      
      // Category colors
      const colors: Record<string, string> = {
        Electronics: '#3b82f6',
        Books: '#8b5cf6',
        Food: '#10b981',
        Clothing: '#f59e0b',
        Sports: '#ef4444',
        Home: '#ec4899',
      };
      
      // Draw shadow for selected/result items
      if (isSelected || isResult) {
        ctx.shadowColor = colors[item.category] || '#6b7280';
        ctx.shadowBlur = isSelected ? 15 : 10;
      }
      
      ctx.fillStyle = colors[item.category] || '#6b7280';
      ctx.beginPath();
      ctx.arc(x, y, isSelected ? 12 : isResult ? 10 : 7, 0, Math.PI * 2);
      ctx.fill();
      
      ctx.shadowBlur = 0;
      
      if (isSelected || isResult) {
        ctx.strokeStyle = isSelected ? '#cc9200' : colors[item.category] + 'aa';
        ctx.lineWidth = isSelected ? 3 : 2;
        ctx.beginPath();
        ctx.arc(x, y, isSelected ? 16 : 14, 0, Math.PI * 2);
        ctx.stroke();
      }
      
      // Label with emoji
      ctx.fillStyle = 'var(--ifm-color-content)';
      ctx.font = isSelected || isResult ? 'bold 13px system-ui' : '12px system-ui';
      ctx.textAlign = 'center';
      ctx.fillText(item.label, x, y - 22);
    });
  }, [items, selectedItem, searchResults]);

  return (
    <div className="rounded-xl border border-solid border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-background-surface-color)] p-6 my-8">
      {/* Status bar */}
      <div className="mb-4 rounded-lg bg-[var(--ifm-code-background)] px-4 py-3 font-mono text-sm">
        <div className="flex items-center gap-2">
          <span className={`inline-block h-2 w-2 rounded-full ${
            isInitializing 
              ? 'bg-yellow-500 animate-pulse' 
              : error 
                ? 'bg-red-500' 
                : db 
                  ? 'bg-green-500' 
                  : 'bg-gray-400'
          }`} />
          <span className="opacity-90">{status}</span>
        </div>
      </div>

      {/* Error display */}
      {error && (
        <div className="mb-4 rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-600 dark:text-red-400">
          <strong>Error:</strong> {error}
        </div>
      )}

      {/* Action button */}
      {!db && (
        <div className="mb-6">
          <button
            onClick={initializeWasm}
            disabled={isInitializing}
            className="rounded-lg bg-gradient-to-r from-[var(--ifm-color-primary)] to-[var(--ifm-color-primary-dark)] px-6 py-2.5 font-semibold text-white shadow-lg transition-all hover:shadow-xl disabled:opacity-50 disabled:cursor-not-allowed">
            {isInitializing ? 'Initializing...' : 'Initialize Database'}
          </button>
        </div>
      )}

      {/* Search Interface */}
      {db && items.length > 0 && (
        <div className="mb-4">
          <div className="relative">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => handleSearchQuery(e.target.value)}
              placeholder="🔍 Search products... (try 'laptop', 'book', 'coffee', 'shoes')"
              className="w-full rounded-lg border-2 border-[var(--ifm-color-primary)] bg-[var(--ifm-background-surface-color)] px-4 py-3 text-lg outline-none focus:border-[var(--ifm-color-primary-dark)] focus:shadow-lg transition-all"
            />
            {searchQuery && (
              <button
                onClick={() => {
                  setSearchQuery('');
                  setSearchResults([]);
                  setSelectedItem(null);
                  setStatus('Ready! Type to search or click any item.');
                }}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 text-xl"
              >
                ×
              </button>
            )}
          </div>
          {searchTime > 0 && (
            <div className="mt-2 text-xs text-[var(--ifm-color-primary)] font-mono">
              ⚡ Search completed in {searchTime.toFixed(2)}ms
            </div>
          )}
        </div>
      )}

      {/* Visualization */}
      {db && items.length > 0 && (
        <div className="space-y-4">
          {/* Canvas */}
          <div className="rounded-lg border border-[var(--ifm-color-emphasis-300)] bg-white dark:bg-[#1a1a1a] p-4">
            <canvas 
              ref={canvasRef}
              width={800}
              height={600}
              className="w-full cursor-pointer"
              onClick={(e) => {
                if (!canvasRef.current) return;
                const rect = canvasRef.current.getBoundingClientRect();
                const x = (e.clientX - rect.left) / rect.width * 800;
                const y = (e.clientY - rect.top) / rect.height * 600;
                
                // Find closest item to click
                let closest: VectorItem | null = null;
                let minDist = Infinity;
                
                const padding = 40;
                const xs = items.map(i => i.x);
                const ys = items.map(i => i.y);
                const minX = Math.min(...xs);
                const maxX = Math.max(...xs);
                const minY = Math.min(...ys);
                const maxY = Math.max(...ys);
                const scaleX = (ix: number) => padding + ((ix - minX) / (maxX - minX)) * (800 - 2 * padding);
                const scaleY = (iy: number) => padding + ((iy - minY) / (maxY - minY)) * (600 - 2 * padding);
                
                items.forEach(item => {
                  const ix = scaleX(item.x);
                  const iy = scaleY(item.y);
                  const dist = Math.sqrt((x - ix) ** 2 + (y - iy) ** 2);
                  if (dist < minDist && dist < 30) {
                    minDist = dist;
                    closest = item;
                  }
                });
                
                if (closest) {
                  searchSimilar(closest);
                }
              }}
            />
            <div className="mt-3 flex flex-wrap gap-3 text-xs">
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#3b82f6] shadow"></span>
                <span>Electronics</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#8b5cf6] shadow"></span>
                <span>Books</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#10b981] shadow"></span>
                <span>Food</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#f59e0b] shadow"></span>
                <span>Clothing</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#ef4444] shadow"></span>
                <span>Sports</span>
              </div>
              <div className="flex items-center gap-1.5">
                <span className="inline-block w-3 h-3 rounded-full bg-[#ec4899] shadow"></span>
                <span>Home</span>
              </div>
            </div>
          </div>

          {/* Search Results */}
          {selectedItem && searchResults.length > 0 && (
            <div className="rounded-lg border border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-code-background)] p-4">
              <h4 className="font-semibold mb-3">Similar to "{selectedItem.label}":</h4>
              <div className="space-y-2">
                {searchResults.map(({ item, similarity }) => (
                  <div key={item.id} className="flex items-center justify-between p-2 rounded bg-[var(--ifm-background-surface-color)]">
                    <div className="flex items-center gap-2">
                      <span className="font-medium">{item.label}</span>
                      <span className="text-xs opacity-60">{item.category}</span>
                    </div>
                    <span className="text-sm font-mono opacity-75">{(similarity * 100).toFixed(1)}%</span>
                  </div>
                ))}
              </div>
            </div>
          )}

          {/* Product List */}
          <details className="rounded-lg border border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-code-background)] p-4">
            <summary className="font-semibold cursor-pointer">All Items in Store ({items.length})</summary>
            <div className="mt-3 grid grid-cols-3 gap-2">
              {items.map(item => (
                <button
                  key={item.id}
                  onClick={() => searchSimilar(item)}
                  className="p-2 rounded text-left text-sm bg-[var(--ifm-background-surface-color)] hover:bg-[var(--ifm-color-emphasis-100)] transition-colors">
                  <div className="font-medium">{item.label}</div>
                  <div className="text-xs opacity-60">{item.category}</div>
                </button>
              ))}
            </div>
          </details>
        </div>
      )}

      {/* Info note */}
      <div className="mt-6 rounded-lg border-l-4 border-[var(--ifm-color-primary)] bg-[var(--ifm-code-background)] px-4 py-3 text-sm opacity-90">
        <p className="font-semibold mb-1">How it works</p>
        <p className="text-xs opacity-75">
          This demo uses multi-threaded WASM to store and search vectors. Click any item to find similar ones using cosine similarity. 
          The visualization shows a 2D projection of high-dimensional (128-dim) vectors.
        </p>
      </div>
    </div>
  );
}

export default function WasmDemo() {
  return (
    <BrowserOnly fallback={<div className="rounded-lg bg-[var(--ifm-code-background)] p-8 text-center opacity-75">Loading WASM demo...</div>}>
      {() => <WasmDemoInner />}
    </BrowserOnly>
  );
}
