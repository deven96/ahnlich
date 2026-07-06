import {useEffect, useState, type ReactNode} from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';

import {GithubIcon} from '../icons';

const NAV_LINKS = [
  {label: 'Docs', to: '/docs/getting-started'},
  {label: 'Guides', to: '/docs/guides'},
  {label: 'Blog', to: '/blog'},
];

/**
 * Marketing-style navbar used only on the landing page. It floats over the
 * dark hero, is transparent at the top of the page, and gains a blurred
 * backdrop once the user scrolls. Docs/blog keep the default Docusaurus navbar.
 */
export default function HomeNavbar(): ReactNode {
  const logo = useBaseUrl('/img/logo.png');
  const [scrolled, setScrolled] = useState(false);
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 24);
    onScroll();
    window.addEventListener('scroll', onScroll, {passive: true});
    return () => window.removeEventListener('scroll', onScroll);
  }, []);

  return (
    <nav
      className={`fixed inset-x-0 top-0 z-50 transition-colors duration-300 ${
        scrolled
          ? 'border-b border-solid border-[#e0eaef] bg-white/80 backdrop-blur dark:border-white/10 dark:bg-[#08161d]/80'
          : 'border-b border-solid border-transparent bg-transparent'
      }`}>
      <div className="container flex h-16 items-center justify-between">
        <Link
          to="/"
          className="flex items-center gap-2 text-[#0c1e28] no-underline hover:text-[#0c1e28] dark:text-white dark:hover:text-white">
          <img src={logo} alt="Ahnlich logo" className="h-8 w-8" />
          <span className="text-lg font-bold tracking-tight">AHNLICH</span>
        </Link>

        {/* desktop links */}
        <div className="hidden items-center gap-8 md:flex">
          {NAV_LINKS.map((link) => (
            <Link
              key={link.label}
              to={link.to}
              className="text-sm font-medium text-[#45585f] no-underline transition-colors hover:text-[#0c1e28] dark:text-white/70 dark:hover:text-white">
              {link.label}
            </Link>
          ))}
          <Link
            href="https://github.com/deven96/ahnlich"
            aria-label="GitHub"
            title="GitHub"
            className="inline-flex h-9 w-9 items-center justify-center rounded-full border border-solid border-[#cdddE4] bg-white text-[#0c1e28] no-underline shadow-sm transition-colors hover:border-primary/50 hover:text-primary dark:border-white/15 dark:bg-white/5 dark:text-white dark:shadow-none dark:backdrop-blur dark:hover:border-secondary/50 dark:hover:text-secondary [&_svg]:h-[18px] [&_svg]:w-[18px]">
            <GithubIcon />
          </Link>
        </div>

        {/* mobile toggle */}
        <button
          aria-label="Toggle navigation"
          onClick={() => setOpen((v) => !v)}
          className="flex h-9 w-9 items-center justify-center rounded-lg border border-solid border-[#cdddE4] bg-white text-[#0c1e28] dark:border-white/15 dark:bg-white/5 dark:text-white md:hidden">
          <svg
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth={2}
            strokeLinecap="round"
            className="h-5 w-5">
            {open ? (
              <path d="M18 6 6 18M6 6l12 12" />
            ) : (
              <path d="M4 6h16M4 12h16M4 18h16" />
            )}
          </svg>
        </button>
      </div>

      {/* mobile menu */}
      {open && (
        <div className="border-t border-solid border-[#e0eaef] bg-white/95 backdrop-blur dark:border-white/10 dark:bg-[#08161d]/95 md:hidden">
          <div className="container flex flex-col gap-1 py-4">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.label}
                to={link.to}
                onClick={() => setOpen(false)}
                className="rounded-lg px-3 py-2 text-sm font-medium text-[#45585f] no-underline transition-colors hover:bg-[#f4f8fa] hover:text-[#0c1e28] dark:text-white/80 dark:hover:bg-white/5 dark:hover:text-white">
                {link.label}
              </Link>
            ))}
            <Link
              href="https://github.com/deven96/ahnlich"
              onClick={() => setOpen(false)}
              className="flex items-center gap-2 rounded-lg px-3 py-2 text-sm font-medium text-[#45585f] no-underline transition-colors hover:bg-[#f4f8fa] hover:text-[#0c1e28] dark:text-white/80 dark:hover:bg-white/5 dark:hover:text-white [&_svg]:h-[18px] [&_svg]:w-[18px]">
              <GithubIcon />
              GitHub
            </Link>
          </div>
        </div>
      )}
    </nav>
  );
}
