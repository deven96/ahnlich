import {useEffect, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';

const REPO = 'deven96/ahnlich';

function fmt(n: number): string {
  return n >= 1000 ? (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k' : String(n);
}

function GithubMark(): ReactNode {
  return (
    <svg viewBox="0 0 24 24" width="18" height="18" fill="currentColor" aria-hidden>
      <path d="M12 .5C5.7.5.5 5.7.5 12a11.5 11.5 0 0 0 7.9 10.9c.6.1.8-.2.8-.5v-2c-3.2.7-3.9-1.4-3.9-1.4-.5-1.3-1.3-1.7-1.3-1.7-1-.7.1-.7.1-.7 1.2.1 1.8 1.2 1.8 1.2 1 1.8 2.8 1.3 3.5 1 .1-.8.4-1.3.7-1.6-2.6-.3-5.3-1.3-5.3-5.7 0-1.3.5-2.3 1.2-3.1-.1-.3-.5-1.5.1-3.1 0 0 1-.3 3.3 1.2a11.5 11.5 0 0 1 6 0C17.3 4.7 18.3 5 18.3 5c.6 1.6.2 2.8.1 3.1.8.8 1.2 1.8 1.2 3.1 0 4.4-2.7 5.4-5.3 5.7.4.4.8 1.1.8 2.2v3.3c0 .3.2.6.8.5A11.5 11.5 0 0 0 23.5 12C23.5 5.7 18.3.5 12 .5Z" />
    </svg>
  );
}

const iconLine = {
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.7,
  strokeLinecap: 'round' as const,
  strokeLinejoin: 'round' as const,
};

/**
 * Compact GitHub repo card shown at the top of the docs sidebar — repo name plus
 * live star and fork counts. Falls back to a dash while loading / if rate-limited.
 */
export default function GithubRepoCard(): ReactNode {
  const [data, setData] = useState<{stars: number; forks: number} | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetch(`https://api.github.com/repos/${REPO}`)
      .then((r) => (r.ok ? r.json() : null))
      .then((d) => {
        if (!cancelled && d && typeof d.stargazers_count === 'number') {
          setData({stars: d.stargazers_count, forks: d.forks_count});
        }
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <Link href={`https://github.com/${REPO}`} className="ahn-repo-card" aria-label={`${REPO} on GitHub`}>
      <span className="ahn-repo-card__top">
        <GithubMark />
        <span className="ahn-repo-card__name">{REPO}</span>
      </span>
      <span className="ahn-repo-card__stats">
        <span className="ahn-repo-card__stat">
          <svg viewBox="0 0 24 24" width="14" height="14" {...iconLine} aria-hidden>
            <path d="m12 2.5 2.9 5.9 6.5.9-4.7 4.6 1.1 6.5L12 17.8 6.2 20.9l1.1-6.5L2.6 9.3l6.5-.9L12 2.5z" />
          </svg>
          {data ? fmt(data.stars) : '—'}
        </span>
        <span className="ahn-repo-card__stat">
          <svg viewBox="0 0 24 24" width="14" height="14" {...iconLine} aria-hidden>
            <circle cx="6" cy="4" r="2" />
            <circle cx="18" cy="4" r="2" />
            <circle cx="12" cy="20" r="2" />
            <path d="M6 6v2a2 2 0 0 0 2 2h8a2 2 0 0 0 2-2V6M12 12v6" />
          </svg>
          {data ? fmt(data.forks) : '—'}
        </span>
      </span>
    </Link>
  );
}
