import React from 'react';
import BrowserOnly from '@docusaurus/BrowserOnly';
import { useBookSearch } from '../BookSearchDemo/BookSearchCore';

function HomepageSearchDemoInner() {
  const [state, actions] = useBookSearch();
  const { db, searchQuery, searchResults, error, searchTime, queryEmbeddingTime } = state;
  const { handleSearch } = actions;

  return (
    <div className="w-full">
      {/* Search bar - always visible */}
      <div className="relative mb-6">
        <div className="absolute -inset-1 rounded-2xl bg-gradient-to-r from-primary/30 to-secondary/30 opacity-75 blur-lg"></div>
        <input
          type="text"
          value={searchQuery}
          onChange={(e) => handleSearch(e.target.value)}
          placeholder={db ? "🔍 Search 1,000 passages from Orwell's works: equality, rebellion, poverty, war..." : 'Loading search engine...'}
          disabled={!db}
          className="relative w-full rounded-xl border-2 border-primary/30 bg-white px-6 py-4 text-lg outline-none shadow-xl transition-all placeholder:text-sm placeholder:text-gray-400 focus:border-primary focus:shadow-2xl disabled:cursor-wait disabled:opacity-50 dark:bg-[#0a1a22] dark:text-white dark:placeholder:text-gray-500"
          autoFocus={!!db}
        />
        {!db && (
          <div className="absolute right-4 top-1/2 -translate-y-1/2">
            <div className="h-5 w-5 animate-spin rounded-full border-2 border-primary border-r-transparent" />
          </div>
        )}
      </div>

      {/* Search results */}
      {searchResults.length > 0 && (
        <div className="space-y-3">
          <div className="text-xs font-mono opacity-60">
            ⚡ {searchTime.toFixed(0)}ms ({queryEmbeddingTime.toFixed(0)}ms AI + {(searchTime - queryEmbeddingTime).toFixed(0)}ms search) • {searchResults.length} results from 1,000 passages
          </div>
          
          {searchResults.map(({ sentence, similarity }, idx) => (
            <div 
              key={sentence.id}
              className="group rounded-lg border border-gray-200/50 bg-white/60 p-4 backdrop-blur-sm transition-all hover:border-primary/30 hover:shadow-md dark:border-gray-700/50 dark:bg-[#0a1a22]/60"
            >
              <div className="flex items-start gap-3">
                <div className="flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-full bg-gradient-to-br from-primary to-secondary text-sm font-bold text-white">
                  {idx + 1}
                </div>
                <div className="flex-1 text-sm leading-relaxed line-clamp-3 group-hover:line-clamp-none transition-all">
                  {sentence.text}
                </div>
                <div className="flex items-center gap-2">
                  <div className="h-1.5 w-16 overflow-hidden rounded-full bg-gray-200 dark:bg-gray-700">
                    <div 
                      className="h-full bg-gradient-to-r from-yellow-400 via-green-500 to-green-600 transition-all" 
                      style={{ width: `${similarity * 100}%` }}
                    />
                  </div>
                  <span className="font-mono text-xs font-bold text-primary">
                    {Math.round(similarity * 100)}%
                  </span>
                </div>
              </div>
              <div className="mt-2 flex items-center gap-2 text-xs opacity-60">
                <span className="font-medium">{sentence.book}</span>
                {sentence.chapter && <span>• Ch. {sentence.chapter}</span>}
              </div>
            </div>
          ))}
        </div>
      )}



      {/* Error state */}
      {error && (
        <div className="rounded-lg border border-red-500/30 bg-red-500/10 px-4 py-3 text-sm text-red-600 dark:text-red-400">
          <strong>Error:</strong> {error}
        </div>
      )}
    </div>
  );
}

export default function HomepageSearchDemo() {
  return (
    <BrowserOnly fallback={<div className="text-center py-8 opacity-60">Loading search demo...</div>}>
      {() => <HomepageSearchDemoInner />}
    </BrowserOnly>
  );
}
