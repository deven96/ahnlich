import { useEffect, useState, useRef } from 'react';

export interface Sentence {
  id: string;
  text: string;
  vector: number[];
  book: string;
  chapter?: number;
}

export interface SearchResult {
  sentence: Sentence;
  similarity: number;
}

export interface BookSearchState {
  db: any;
  sentences: Sentence[];
  searchQuery: string;
  searchResults: SearchResult[];
  error: string | null;
  searchTime: number;
  queryEmbeddingTime: number;
  embeddingTime: number;
  insertTime: number;
  modelLoaded: boolean;
  isInitializing: boolean;
  status: string;
}

export interface BookSearchActions {
  setSearchQuery: (query: string) => void;
  handleSearch: (query: string) => Promise<void>;
}

export function useBookSearch(): [BookSearchState, BookSearchActions] {
  const [status, setStatus] = useState<string>('Loading...');
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

  // Preload the embedding model on mount, then auto-initialize database
  useEffect(() => {
    async function preloadAndInitialize() {
      try {
        // Check if SharedArrayBuffer is available (requires COOP/COEP headers)
        // If not available on first load, force a full page reload to get headers
        if (typeof SharedArrayBuffer === 'undefined') {
          // Check if we've already tried reloading (prevent infinite loop)
          const reloadAttempted = sessionStorage.getItem('sab-reload-attempted');
          if (!reloadAttempted) {
            sessionStorage.setItem('sab-reload-attempted', 'true');
            window.location.reload();
            return;
          } else {
            // Reload didn't help - server doesn't have headers configured
            throw new Error('SharedArrayBuffer not available. COOP/COEP headers missing.');
          }
        }
        
        // Clear the reload flag on successful load
        sessionStorage.removeItem('sab-reload-attempted');
        
        setStatus('Loading embedding model (one-time)...');
        const { pipeline, env } = await import('@huggingface/transformers');
        
        env.allowRemoteModels = true;
        env.allowLocalModels = false;
        
        embeddingModel.current = await pipeline('feature-extraction', 'Xenova/all-MiniLM-L6-v2');
        setModelLoaded(true);
        
        // Auto-initialize database after model loads
        await initialize();
      } catch (err) {
        setError('Failed to load: ' + (err instanceof Error ? err.message : String(err)));
        setStatus('Loading failed. Please refresh the page.');
      }
    }
    
    preloadAndInitialize();
  }, []);

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
      setStatus('Loading sentences with pre-computed embeddings...');
      const response = await fetch('/sentencesWithEmbeddings.json');
      const allSentences = await response.json();
      
      const embStart = performance.now();
      setStatus(`Loading ${allSentences.length} passages...`);
      
      const dimension = allSentences[0].embedding.length;
      
      const sentenceObjs: Sentence[] = allSentences.map((sent, idx) => ({
        id: `sent-${idx}`,
        text: sent.text,
        vector: sent.embedding,
        book: sent.book,
        chapter: sent.chapter
      }));
      
      const embEnd = performance.now();
      setEmbeddingTime(embEnd - embStart);
      
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
      return;
    }
    
    const startTime = performance.now();
    setStatus(`Searching for "${query}"...`);
    
    try {
      const protobufModule = await import('/wasm-pkg/protobuf-bundle.js');
      const protobuf = protobufModule.default;
      const { queryPb, serverPb } = protobuf;
      const StoreKey = protobuf.StoreKey;
      
      const embStart = performance.now();
      const queryVector = await createEmbedding(query);
      const embEnd = performance.now();
      setQueryEmbeddingTime(embEnd - embStart);
      
      const searchReq = new queryPb.GetSimN({
        store: 'book',
        searchInput: new StoreKey({ key: queryVector }),
        closestN: 5,
        algorithm: 2 // CosineSimilarity
      });
      
      const resultBytes = db.get_sim_n(searchReq.toBinary());
      const response = serverPb.GetSimN.fromBinary(new Uint8Array(resultBytes));
      
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

  return [
    {
      db,
      sentences,
      searchQuery,
      searchResults,
      error,
      searchTime,
      queryEmbeddingTime,
      embeddingTime,
      insertTime,
      modelLoaded,
      isInitializing,
      status,
    },
    {
      setSearchQuery,
      handleSearch,
    },
  ];
}
