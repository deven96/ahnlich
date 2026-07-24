import {
  useCallback,
  useEffect,
  useRef,
  useState,
  type ReactNode,
} from 'react';
import {createPortal} from 'react-dom';
import OriginalSearchBar from '@theme-original/SearchBar';
import type SearchBarType from '@theme/SearchBar';
import type {WrapperProps} from '@docusaurus/types';
import useIsBrowser from '@docusaurus/useIsBrowser';

type Props = WrapperProps<typeof SearchBarType>;

function SearchIcon() {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden>
      <circle cx="11" cy="11" r="8" />
      <path d="m21 21-4.3-4.3" />
    </svg>
  );
}

/**
 * Command-palette search. The navbar shows a trigger button; clicking it — or
 * pressing ⌘K / Ctrl-K — opens a centered modal that hosts the real search
 * input and its live results. Esc or a backdrop click closes it.
 */
export default function SearchBarWrapper(props: Props): ReactNode {
  const isBrowser = useIsBrowser();
  const [open, setOpen] = useState(false);
  const modalRef = useRef<HTMLDivElement>(null);
  const isMac =
    isBrowser && /mac|iphone|ipad/i.test(navigator.platform || navigator.userAgent);

  const close = useCallback(() => setOpen(false), []);

  // ⌘K / Ctrl-K toggles, Esc closes
  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key.toLowerCase() === 'k') {
        e.preventDefault();
        setOpen((o) => !o);
      } else if (e.key === 'Escape') {
        setOpen(false);
      }
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, []);

  // focus the real input + lock body scroll while open
  useEffect(() => {
    if (!open) return;
    document.body.style.overflow = 'hidden';
    const t = window.setTimeout(() => {
      modalRef.current
        ?.querySelector<HTMLInputElement>('.navbar__search-input')
        ?.focus();
    }, 40);
    return () => {
      window.clearTimeout(t);
      document.body.style.overflow = '';
    };
  }, [open]);

  return (
    <>
      <button
        type="button"
        className="ahn-search-trigger"
        onClick={() => setOpen(true)}
        aria-label="Search">
        <SearchIcon />
        <span className="ahn-search-trigger__label">Search docs…</span>
        <span className="ahn-search-trigger__kbd">
          <kbd>{isMac ? '⌘' : 'Ctrl'}</kbd>
          <kbd>K</kbd>
        </span>
      </button>

      {isBrowser &&
        open &&
        createPortal(
          <div
            className="ahn-search-overlay"
            role="dialog"
            aria-modal="true"
            aria-label="Search docs"
            onMouseDown={(e) => {
              if (e.target === e.currentTarget) close();
            }}>
            <div className="ahn-search-modal" ref={modalRef}>
              <OriginalSearchBar {...props} />
            </div>
          </div>,
          document.body,
        )}
    </>
  );
}
