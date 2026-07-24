import React from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';
import { useBookSearch } from './BookSearchCore';

function BookSearchDemoInner() {
  const [state, actions] = useBookSearch();
  const { 
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
    status 
  } = state;
  const { setSearchQuery, handleSearch } = actions;

  return (
    <div className="rounded-xl border border-solid border-[var(--ifm-color-emphasis-300)] bg-[var(--ifm-background-surface-color)] p-6 my-8">
      {/* Header */}
      <div className="mb-6">
        <h3 className="text-2xl font-bold mb-2">📚 Semantic Book Search</h3>
        <p className="text-sm opacity-75">
          Search 1,000 passages across 4 George Orwell books using vector similarity — running entirely in your browser!
        </p>
        <p className="text-xs opacity-60 mt-1">
          Books: Animal Farm • 1984 • Homage to Catalonia • Down and Out in Paris and London
        </p>
        <p className="text-xs opacity-60">
          Embeddings: Transformers.js (all-MiniLM-L6-v2) • Vector DB: Ahnlich WASM with multi-threading
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

      {/* Loading state */}
      {!db && (isInitializing || !modelLoaded) && (
        <div className="text-center py-8">
          <div className="inline-block h-8 w-8 animate-spin rounded-full border-4 border-solid border-current border-r-transparent align-[-0.125em] motion-reduce:animate-[spin_1.5s_linear_infinite]" role="status">
            <span className="!absolute !-m-px !h-px !w-px !overflow-hidden !whitespace-nowrap !border-0 !p-0 ![clip:rect(0,0,0,0)]">Loading...</span>
          </div>
          <p className="mt-4 text-sm opacity-70">Initializing search engine...</p>
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
