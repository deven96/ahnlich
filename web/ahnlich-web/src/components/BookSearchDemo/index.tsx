import React, { useEffect, useState, useRef } from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';

interface Sentence {
  id: string;
  text: string;
  vector: number[];
  book: string;
  chapter?: number;
}

interface SearchResult {
  sentence: Sentence;
  similarity: number;
}

function BookSearchDemoInner() {
  const [status, setStatus] = useState<string>('Loading embedding model...');
  const [db, setDb] = useState<any>(null);
  const [sentences, setSentences] = useState<Sentence[]>([]);
  const [searchQuery, setSearchQuery] = useState<string>('');
  const [searchResults, setSearchResults] = useState<SearchResult[]>([]);
  const [isInitializing, setIsInitializing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [searchTime, setSearchTime] = useState<number>(0);
  const [queryEmbeddingTime, setQueryEmbeddingTime] = useState<number>(0);
  const [embeddingTime, setEmbeddingTime] = useState<number>(0);
  const [insertTime, setInsertTime] = useState<number>(0);
  const [modelLoaded, setModelLoaded] = useState(false);
  const embeddingModel = useRef<any>(null);

  // Preload the embedding model on mount
  useEffect(() => {
    async function preloadModel() {
      try {
        setStatus('Loading embedding model (one-time)...');
        const { pipeline, env } = await import('@huggingface/transformers');
        
        env.allowRemoteModels = true;
        env.allowLocalModels = false;
        
        embeddingModel.current = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2');
        setModelLoaded(true);
        setStatus('Ready! Click "Initialize Database" to start.');
      } catch (err) {
        setError('Failed to load embedding model: ' + (err instanceof Error ? err.message : String(err)));
        setStatus('Model loading failed. Click Initialize to retry.');
      }
    }
    
    preloadModel();
  }, []);

  // Create embedding using Transformers.js (preloaded on mount)
  async function createEmbedding(text: string): Promise<number[]> {
    if (!embeddingModel.current) {
      throw new Error('Embedding model not loaded. Please wait for initialization.');
    }
    
    const output = await embeddingModel.current(text, { pooling: 'mean', normalize: true });
    return Array.from(output.data);
  }

  async function initialize() {
    setIsInitializing(true);
    setError(null);
    
    // Wait for model to load if it hasn't finished yet
    if (!modelLoaded) {
      setStatus('Waiting for embedding model to load...');
      while (!embeddingModel.current && !error) {
        await new Promise(resolve => setTimeout(resolve, 100));
      }
      if (error) {
        setIsInitializing(false);
        return;
      }
    }
    
    try {
      setStatus('Loading WASM module...');
      const wasmModule = await import('/wasm-pkg/ahnlich_wasm_db.js');
      const { default: init, initThreadPool, AhnlichDB } = wasmModule;
      
      await init();
      setStatus('Initializing multi-threading...');
      
      const numThreads = Math.min(navigator.hardwareConcurrency || 4, 8);
      try {
        await initThreadPool(numThreads);
        setStatus(`Threads initialized (${numThreads} workers)`);
      } catch (threadErr) {
        throw new Error('Thread pool initialization failed. COOP/COEP headers required.');
      }
      
      const dbInstance = new AhnlichDB();
      // Fetch pre-computed embeddings
      setStatus('Loading sentences with pre-computed embeddings...');
      const response = await fetch('/sentencesWithEmbeddings.json');
      const allSentences = await response.json();
      
      const embStart = performance.now();
      setStatus(`Loading ${allSentences.length} passages...`);
      
      const dimension = allSentences[0].embedding.length;
      
      // Map pre-computed embeddings to sentence objects
      const sentenceObjs: Sentence[] = allSentences.map((sent, idx) => ({
        id: `sent-${idx}`,
        text: sent.text,
        vector: sent.embedding,
        book: sent.book,
        chapter: sent.chapter
      }));
      
      const embEnd = performance.now();
      setEmbeddingTime(embEnd - embStart);
      
      // Create vector store
      const insertStart = performance.now();
      setStatus('Creating vector store...');
      const protobufModule = await import('/wasm-pkg/protobuf-bundle.js');
      const protobuf = protobufModule.default;
      const { queryPb } = protobuf;
      
      const createReq = new queryPb.CreateStore({
        store: 'book',
        dimension,
        createPredicates: ['book', 'chapter'],
        nonLinearIndices: [],
        errorIfExists: false,
      });
      dbInstance.create_store(createReq.toBinary());
      
      // Insert sentences in batches
      const StoreKey = protobuf.StoreKey;
      const StoreValue = protobuf.StoreValue;
      const DbStoreEntry = protobuf.DbStoreEntry;
      const MetadataValue = protobuf.MetadataValue;
      
      const insertBatchSize = 50;
      for (let i = 0; i < sentenceObjs.length; i += insertBatchSize) {
        const batch = sentenceObjs.slice(i, i + insertBatchSize);
        const progress = Math.floor((i / sentenceObjs.length) * 100);
        setStatus(`Inserting into database (${progress}%)...`);
        
        const setReq = new queryPb.Set({
          store: 'book',
          inputs: batch.map(sent => new DbStoreEntry({
            key: new StoreKey({ key: sent.vector }),
            value: new StoreValue({
              value: {
                id: new MetadataValue({ value: { case: 'rawString', value: sent.id } }),
                text: new MetadataValue({ value: { case: 'rawString', value: sent.text } }),
                book: new MetadataValue({ value: { case: 'rawString', value: sent.book } }),
                ...(sent.chapter && { chapter: new MetadataValue({ value: { case: 'rawString', value: sent.chapter.toString() } }) }),
              }
            })
          }))
        });
        dbInstance.set(setReq.toBinary());
      }
      
      const insertEnd = performance.now();
      setInsertTime(insertEnd - insertStart);
      
      setDb(dbInstance);
      setSentences(sentenceObjs);
      setStatus(`Ready! Indexed ${sentenceObjs.length} sentences from 4 books. Start searching!`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('Initialization failed');
    } finally {
      setIsInitializing(false);
    }
  }

  async function handleSearch(query: string) {
    setSearchQuery(query);
    
    if (!query.trim() || !db || sentences.length === 0) {
      setSearchResults([]);
      setStatus('Ready! Type a query to search Animal Farm.');
      return;
    }
    
    const startTime = performance.now();
    setStatus(`Searching for "${query}"...`);
    
    try {
      const protobufModule = await import('/wasm-pkg/protobuf-bundle.js');
      const protobuf = protobufModule.default;
      const { queryPb, serverPb } = protobuf;
      const StoreKey = protobuf.StoreKey;
      
      // Create embedding for query
      const embStart = performance.now();
      const queryVector = await createEmbedding(query);
      const embEnd = performance.now();
      setQueryEmbeddingTime(embEnd - embStart);
      
      // Search
      const searchReq = new queryPb.GetSimN({
        store: 'book',
        searchInput: new StoreKey({ key: queryVector }),
        closestN: 5,
        algorithm: 2 // CosineSimilarity
      });
      
      const resultBytes = db.get_sim_n(searchReq.toBinary());
      const response = serverPb.GetSimN.fromBinary(new Uint8Array(resultBytes));
      
      // Match results
      const results: SearchResult[] = [];
      if (response.entries && response.entries.length > 0) {
        for (const entry of response.entries) {
          const matchedSentence = sentences.find(s => {
            if (!entry.key?.key) return false;
            const entryVec = Array.from(entry.key.key);
            return s.vector.every((v, idx) => Math.abs(v - entryVec[idx]) < 0.0001);
          });
          
          if (matchedSentence) {
            const simValue = entry.similarity?.value ?? 0;
            results.push({ sentence: matchedSentence, similarity: simValue });
          }
        }
      }
      
      const endTime = performance.now();
      setSearchTime(endTime - startTime);
      setSearchResults(results);
      setStatus(`Found ${results.length} results in ${(endTime - startTime).toFixed(2)}ms`);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setStatus('Search failed');
    }
  }

  return (
    <div className="rounded-xl border border-solid border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-background-surface-color)] p-6 my-8">
      {/* Header */}
      <div className="mb-6">
        <h3 className="text-2xl font-bold mb-2">📚 Semantic Book Search</h3>
        <p className="text-sm opacity-75">
          Search 1,000 passages across 4 George Orwell books using vector similarity
        </p>
        <p className="text-xs opacity-60 mt-1">
          Books: Animal Farm • 1984 • Homage to Catalonia • Down and Out in Paris and London
        </p>
        <p className="text-xs opacity-60">
          Embeddings: Transformers.js (all-MiniLM-L6-v2) • Vector DB: Ahnlich WASM
        </p>
      </div>

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

      {/* Initialize button */}
      {!db && !isInitializing && (
        <div className="text-center py-8">
          <button
            onClick={initialize}
            className="rounded-lg bg-gradient-to-r from-[var(--ifm-color-primary)] to-[var(--ifm-color-primary-dark)] px-8 py-3 font-semibold text-white shadow-lg transition-all hover:shadow-xl">
            Initialize Database
          </button>
        </div>
      )}

      {/* Search Interface */}
      {db && (
        <div>
          <div className="relative mb-6">
            <input
              type="text"
              value={searchQuery}
              onChange={(e) => handleSearch(e.target.value)}
              placeholder='🔍 Try: "equality", "windmill", "rebellion", "pigs"...'
              className="w-full rounded-lg border-2 border-[var(--ifm-color-primary)] bg-[var(--ifm-background-surface-color)] px-4 py-3 text-lg outline-none focus:border-[var(--ifm-color-primary-dark)] focus:shadow-lg transition-all"
              autoFocus
            />
            {searchQuery && (
              <button
                onClick={() => {
                  setSearchQuery('');
                  setSearchResults([]);
                }}
                className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 text-2xl"
              >
                ×
              </button>
            )}
          </div>

          {searchTime > 0 && searchResults.length > 0 && (
            <div className="mb-3 text-xs font-mono opacity-70">
              ⚡ {(queryEmbeddingTime + searchTime).toFixed(1)}ms ({queryEmbeddingTime.toFixed(0)}ms embed + {searchTime.toFixed(1)}ms search)
            </div>
          )}

          {/* Search Results */}
          {searchResults.length > 0 && (
            <div className="space-y-3">
              {searchResults.map(({ sentence, similarity }, idx) => (
                <div 
                  key={sentence.id}
                  className="rounded-lg border border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-code-background)] p-4 hover:border-[var(--ifm-color-primary)] transition-all shadow-sm hover:shadow-md"
                >
                  <div className="flex items-start gap-3">
                    <div className="flex items-center justify-center w-8 h-8 rounded-full bg-gradient-to-br from-[var(--ifm-color-primary)] to-[var(--ifm-color-primary-dark)] text-white font-bold text-sm flex-shrink-0">
                      {idx + 1}
                    </div>
                    <div className="flex-1">
                      <div className="text-sm mb-2 leading-relaxed">{sentence.text}</div>
                      <div className="flex items-center gap-3 text-xs">
                        <span className="font-medium opacity-70">{sentence.book}</span>
                        {sentence.chapter && <span className="opacity-60">• Chapter {sentence.chapter}</span>}
                        <div className="flex items-center gap-2 ml-auto">
                          <div className="h-1.5 w-20 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
                            <div 
                              className="h-full bg-gradient-to-r from-yellow-400 via-green-500 to-green-600 transition-all" 
                              style={{ width: `${similarity * 100}%` }}
                            />
                          </div>
                          <span className="font-mono font-bold text-[var(--ifm-color-primary)]">
                            {Math.round(similarity * 100)}%
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              ))}
            </div>
          )}

          {searchQuery && searchResults.length === 0 && (
            <div className="text-center py-8 opacity-60">
              No results found. Try a different query.
            </div>
          )}

          {/* Index Stats (below results) */}
          {(embeddingTime > 0 || insertTime > 0) && (
            <details className="mt-6">
              <summary className="cursor-pointer text-xs opacity-60 hover:opacity-100">
                📊 Indexing stats ({sentences.length.toLocaleString()} passages)
              </summary>
              <div className="mt-2 text-xs font-mono opacity-70 space-y-1 ml-4">
                <div>• Load embeddings: {(embeddingTime / 1000).toFixed(3)}s</div>
                <div>• Insert to Ahnlich: {(insertTime / 1000).toFixed(2)}s</div>
              </div>
            </details>
          )}
        </div>
      )}
    </div>
  );
}

export default function BookSearchDemo() {
  return (
    <BrowserOnly fallback={<div>Loading...</div>}>
      {() => <BookSearchDemoInner />}
    </BrowserOnly>
  );
}
