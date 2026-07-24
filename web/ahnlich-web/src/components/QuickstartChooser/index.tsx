import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';

const ACCENT = 'var(--ifm-color-primary)';
const INK = 'var(--ifm-color-content)';
const MUTED = 'var(--ifm-color-content-secondary)';
const LINE = 'var(--ifm-toc-border-color)';
const SURFACE = 'var(--ifm-background-surface-color)';

const line = {
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.7,
  strokeLinecap: 'round' as const,
  strokeLinejoin: 'round' as const,
};

function SparklesIcon(): ReactNode {
  return (
    <svg viewBox="0 0 24 24" {...line} aria-hidden>
      <path d="M12 3v4M12 17v4M3 12h4M17 12h4" />
      <path d="M12 8.5 13.4 11 16 12l-2.6 1L12 15.5 10.6 13 8 12l2.6-1L12 8.5z" />
    </svg>
  );
}

function DatabaseIcon(): ReactNode {
  return (
    <svg viewBox="0 0 24 24" {...line} aria-hidden>
      <ellipse cx="12" cy="5" rx="8" ry="3" />
      <path d="M4 5v14a8 3 0 0 0 16 0V5" />
      <path d="M4 12a8 3 0 0 0 16 0" />
    </svg>
  );
}

function Arrow(): ReactNode {
  return (
    <svg viewBox="0 0 24 24" {...line} strokeWidth={2.2} aria-hidden>
      <path d="m9 6 6 6-6 6" />
    </svg>
  );
}

/** Flow: two ways in (text→embed, or your own vector) converge on store & search. */
function QuickstartFlow(): ReactNode {
  const chip = (x: number, y: number, w: number, text: string, accent = false) => (
    <g>
      <rect x={x} y={y} width={w} height="42" rx="10" fill={accent ? ACCENT : SURFACE}
        opacity={accent ? 0.12 : 1} stroke={accent ? ACCENT : LINE} strokeWidth={accent ? 1.5 : 1} />
      <text x={x + w / 2} y={y + 26} textAnchor="middle" fontSize="13" fill={INK}>{text}</text>
    </g>
  );
  const arrow = (x1: number, x2: number, y: number) => (
    <path d={`M${x1} ${y} H${x2}`} stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#qs-a)" />
  );
  return (
    <figure style={{margin: '1.25rem 0 0'}}>
      <svg viewBox="0 0 720 210" role="img"
        aria-label="Ahnlich AI takes text or images and embeds them; Ahnlich DB takes your own vectors. Both store and search."
        style={{width: '100%', height: 'auto', fontFamily: 'inherit'}}>
        <defs>
          <marker id="qs-a" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
            <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
          </marker>
        </defs>

        {/* store & search (shared target) */}
        <rect x="556" y="55" width="150" height="100" rx="14" fill={ACCENT} opacity="0.10" />
        <rect x="556" y="55" width="150" height="100" rx="14" fill="none" stroke={ACCENT} strokeWidth="1.5" />
        <text x="631" y="100" textAnchor="middle" fontSize="14" fontWeight="700" fill={INK}>Store &amp;</text>
        <text x="631" y="120" textAnchor="middle" fontSize="14" fontWeight="700" fill={INK}>search</text>

        {/* AI lane */}
        <text x="24" y="46" fontSize="11" fontWeight="700" letterSpacing="0.12em" fill={ACCENT}>AHNLICH AI</text>
        {chip(24, 58, 150, 'Text / image')}
        {arrow(178, 214, 79)}
        {chip(216, 58, 150, 'AI embeds', true)}
        {arrow(370, 552, 79)}

        {/* DB lane */}
        <text x="24" y="140" fontSize="11" fontWeight="700" letterSpacing="0.12em" fill={MUTED}>AHNLICH DB</text>
        {chip(24, 152, 150, 'Your vectors')}
        {arrow(178, 552, 173)}
        <text x="300" y="168" fontSize="12" fill={MUTED}>sent directly</text>
      </svg>
    </figure>
  );
}

export default function QuickstartChooser(): ReactNode {
  return (
    <>
      <QuickstartFlow />
      <div className="ahn-qs-grid">
        <Link to="/docs/getting-started/quickstart-ai" className="ahn-qs-card ahn-qs-card--rec">
          <span className="ahn-qs-badge">Recommended</span>
          <span className="ahn-qs-icon"><SparklesIcon /></span>
          <span className="ahn-qs-title">Ahnlich AI</span>
          <span className="ahn-qs-desc">
            Send plain <strong>text</strong> or images — Ahnlich turns them into
            vectors for you. Nothing to know about embeddings or dimensions.
          </span>
          <span className="ahn-qs-start">Start with AI <Arrow /></span>
        </Link>

        <Link to="/docs/getting-started/quickstart-db" className="ahn-qs-card">
          <span className="ahn-qs-icon"><DatabaseIcon /></span>
          <span className="ahn-qs-title">Ahnlich DB</span>
          <span className="ahn-qs-desc">
            Already have embeddings? Bring your own <strong>vectors</strong> and
            store &amp; search them directly.
          </span>
          <span className="ahn-qs-start">Start with DB <Arrow /></span>
        </Link>
      </div>
    </>
  );
}
