import {useEffect, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';

const REPO = 'deven96/ahnlich';

function formatStars(n: number): string {
  if (n >= 1000) return (n / 1000).toFixed(1).replace(/\.0$/, '') + 'k';
  return String(n);
}

/**
 * Live GitHub star count pill. Fetches the repo's stargazer count client-side
 * and links to the repository. Renders a neutral placeholder until loaded.
 */
export default function GitHubStars(): ReactNode {
  const [stars, setStars] = useState<number | null>(null);

  useEffect(() => {
    let cancelled = false;
    fetch(`https://api.github.com/repos/${REPO}`)
      .then((res) => (res.ok ? res.json() : null))
      .then((data) => {
        if (!cancelled && data && typeof data.stargazers_count === 'number') {
          setStars(data.stargazers_count);
        }
      })
      .catch(() => {
        /* offline / rate-limited — keep the placeholder */
      });
    return () => {
      cancelled = true;
    };
  }, []);

  return (
    <Link
      href={`https://github.com/${REPO}`}
      className="inline-flex items-center gap-2 rounded-full border border-solid border-[#e0eaef] bg-white px-4 py-1.5 text-sm font-medium text-[#45585f] no-underline transition-colors hover:border-primary/40 hover:text-[#0c1e28] dark:border-white/15 dark:bg-white/5 dark:text-white/80 dark:hover:border-secondary/50 dark:hover:text-white">
      <svg
        viewBox="0 0 24 24"
        fill="currentColor"
        className="h-4 w-4 text-[#e3b341]"
        aria-hidden>
        <path d="M12 2.5l2.9 5.9 6.5.9-4.7 4.6 1.1 6.5L12 17.8 6.2 20.9l1.1-6.5L2.6 9.3l6.5-.9L12 2.5z" />
      </svg>
      <span className="font-semibold text-[#0c1e28] dark:text-white">
        {stars === null ? '★' : formatStars(stars)}
      </span>
      <span className="text-[#8299a3] dark:text-white/40">Stars on GitHub</span>
    </Link>
  );
}
