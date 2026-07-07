import type {ReactNode} from 'react';

/*
 * Beginner-friendly, theme-aware SVG diagrams for the section Overview pages.
 * Colours come from Infima CSS variables so they adapt to light/dark
 * automatically. Kept deliberately simple — each one tells a first-time
 * reader "here is the whole idea in one picture".
 */

const ACCENT = 'var(--ifm-color-primary)';
const INK = 'var(--ifm-color-content)';
const MUTED = 'var(--ifm-color-content-secondary)';
const LINE = 'var(--ifm-toc-border-color)';
const SURFACE = 'var(--ifm-background-surface-color)';

const wrap = {margin: '1.5rem 0', width: '100%'} as const;

const cap = {
  fontSize: 13,
  color: 'var(--ifm-color-content-secondary)',
  textAlign: 'center',
  marginTop: '0.6rem',
  lineHeight: 1.5,
} as const;

const label = {
  fontSize: 11,
  letterSpacing: '0.1em',
  textTransform: 'uppercase',
  fill: MUTED,
  fontWeight: 700,
} as const;

const mono = {
  fontFamily: 'var(--ifm-font-family-monospace)',
  fontSize: 12,
} as const;

const svgStyle = (max: number) =>
  ({
    width: '100%',
    height: 'auto',
    maxWidth: max,
    display: 'block',
    margin: '0 auto',
    fontFamily: 'inherit',
  } as const);

const fig = (svg: ReactNode, caption: ReactNode): ReactNode => (
  <figure style={wrap}>
    {svg}
    <figcaption style={cap}>{caption}</figcaption>
  </figure>
);

/** Shared arrowhead defs — unique id per diagram to avoid collisions. */
function Arrow({id}: {id: string}): ReactNode {
  return (
    <defs>
      <marker id={id} markerWidth="10" markerHeight="10" refX="7" refY="3" orient="auto">
        <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
      </marker>
    </defs>
  );
}

/**
 * Ahnlich DB in one picture: store a vector + its metadata, then ask for the
 * nearest matches.
 */
export function DbOverviewFigure(): ReactNode {
  return fig(
    <svg viewBox="0 0 640 300" role="img"
      aria-label="Store vectors with metadata, then query returns the nearest matches"
      style={svgStyle(640)}>
      <Arrow id="db-a" />

      {/* ---- STORE row ---- */}
      <text x="20" y="34" style={label}>1 · Store</text>
      {/* input card */}
      <rect x="20" y="46" width="180" height="74" rx="10" fill={SURFACE} stroke={LINE} />
      <text x="34" y="72" style={mono} fill={ACCENT}>[0.12, 0.98, 0.41]</text>
      <text x="34" y="94" style={mono} fill={INK}>genre: "sci-fi"</text>
      <text x="34" y="110" style={mono} fill={MUTED}>a vector + metadata</text>
      {/* arrow to db */}
      <path d="M204 83 H286" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#db-a)" />
      {/* db cylinder */}
      <g>
        <ellipse cx="360" cy="60" rx="60" ry="16" fill={ACCENT} opacity="0.12" />
        <path d="M300 60 V120 A60 16 0 0 0 420 120 V60" fill={ACCENT} opacity="0.08" />
        <ellipse cx="360" cy="60" rx="60" ry="16" fill="none" stroke={ACCENT} strokeWidth="1.5" />
        <path d="M300 60 V120 A60 16 0 0 0 420 120 V60" fill="none" stroke={ACCENT} strokeWidth="1.5" />
        <text x="360" y="150" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Ahnlich DB</text>
        <text x="360" y="168" textAnchor="middle" style={mono} fill={MUTED}>in-memory</text>
      </g>

      {/* ---- SEARCH row ---- */}
      <text x="20" y="210" style={label}>2 · Search</text>
      <rect x="20" y="222" width="180" height="52" rx="10" fill={SURFACE} stroke={LINE} />
      <text x="34" y="245" style={mono} fill={ACCENT}>[0.10, 0.95, 0.44]</text>
      <text x="34" y="264" style={mono} fill={MUTED}>"find similar"</text>
      <path d="M204 248 H296" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#db-a)" />
      <path d="M424 248 H480" stroke={ACCENT} strokeWidth="2" fill="none" markerEnd="url(#db-a)" />

      {/* results, ranked */}
      {[0, 1, 2].map((i) => (
        <g key={i}>
          <rect x="486" y={218 + i * 26} width="140" height="20" rx="5"
            fill={ACCENT} opacity={0.22 - i * 0.06} />
          <text x="496" y={232 + i * 26} style={mono} fill={INK}>match #{i + 1}</text>
        </g>
      ))}
      <text x="486" y="210" style={label}>Nearest</text>
    </svg>,
    'You store each item as a vector plus metadata. To search, you send another vector and Ahnlich DB returns the closest matches, ranked.',
  );
}

/**
 * Ahnlich AI proxy in one picture: you send raw text/images, the proxy turns
 * them into vectors for you and talks to the DB.
 */
export function AiProxyFigure(): ReactNode {
  return fig(
    <svg viewBox="0 0 660 250" role="img"
      aria-label="Your app sends raw text, images or audio; the AI proxy turns them into vectors and talks to the database"
      style={svgStyle(660)}>
      <Arrow id="ai-a" />

      {/* your app */}
      <rect x="16" y="86" width="150" height="78" rx="12" fill={SURFACE} stroke={LINE} />
      <text x="91" y="118" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Your app</text>
      <text x="91" y="138" textAnchor="middle" style={mono} fill={MUTED}>"a red bicycle"</text>
      <text x="91" y="154" textAnchor="middle" style={mono} fill={MUTED}>🖼️ photo · 🔊 audio</text>

      <path d="M170 125 H236" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#ai-a)" />
      <text x="203" y="116" textAnchor="middle" style={label}>raw input</text>

      {/* AI proxy */}
      <rect x="240" y="70" width="176" height="110" rx="12" fill={ACCENT} opacity="0.09" />
      <rect x="240" y="70" width="176" height="110" rx="12" fill="none" stroke={ACCENT} strokeWidth="1.5" />
      <text x="328" y="104" textAnchor="middle" fontSize="14" fontWeight="700" fill={INK}>🧠 Ahnlich AI</text>
      <text x="328" y="126" textAnchor="middle" fontSize="12" fill={MUTED}>picks the model,</text>
      <text x="328" y="143" textAnchor="middle" fontSize="12" fill={MUTED}>turns input into</text>
      <text x="328" y="162" textAnchor="middle" style={mono} fill={ACCENT}>[0.12, 0.98, …]</text>

      <path d="M420 125 H486" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#ai-a)" />
      <text x="453" y="116" textAnchor="middle" style={label}>vectors</text>

      {/* DB cylinder */}
      <ellipse cx="558" cy="98" rx="52" ry="14" fill={ACCENT} opacity="0.12" />
      <path d="M506 98 V150 A52 14 0 0 0 610 150 V98" fill={ACCENT} opacity="0.08" />
      <ellipse cx="558" cy="98" rx="52" ry="14" fill="none" stroke={ACCENT} strokeWidth="1.5" />
      <path d="M506 98 V150 A52 14 0 0 0 610 150 V98" fill="none" stroke={ACCENT} strokeWidth="1.5" />
      <text x="558" y="176" textAnchor="middle" fontSize="12" fontWeight="700" fill={INK}>Ahnlich DB</text>
    </svg>,
    'With the AI proxy you work in text, images, and audio, never numbers. It automatically turns your input into vectors with a machine-learning model, then stores or searches them in Ahnlich DB.',
  );
}

/**
 * Ahnlich CLI in one picture: type DSL commands in a terminal, they go straight
 * to the DB and AI servers.
 */
export function CliFigure(): ReactNode {
  return fig(
    <svg viewBox="0 0 620 260" role="img"
      aria-label="Type commands in a terminal and the CLI sends them to the DB and AI servers"
      style={svgStyle(620)}>
      <Arrow id="cli-a" />

      {/* terminal window */}
      <rect x="16" y="40" width="270" height="180" rx="10" fill={SURFACE} stroke={LINE} />
      <rect x="16" y="40" width="270" height="30" rx="10" fill={ACCENT} opacity="0.1" />
      <circle cx="36" cy="55" r="5" fill={MUTED} opacity="0.5" />
      <circle cx="52" cy="55" r="5" fill={MUTED} opacity="0.5" />
      <circle cx="68" cy="55" r="5" fill={MUTED} opacity="0.5" />
      <text x="150" y="59" textAnchor="middle" style={mono} fill={MUTED}>ahnlich-cli</text>
      <text x="34" y="100" style={mono} fill={ACCENT}>&gt; PING</text>
      <text x="34" y="122" style={mono} fill={INK}>&lt; PONG</text>
      <text x="34" y="150" style={mono} fill={ACCENT}>&gt; CREATESTORE books</text>
      <text x="34" y="172" style={mono} fill={INK}>&lt; OK</text>
      <text x="34" y="200" style={mono} fill={ACCENT}>&gt; GETSIMN 3 …</text>

      {/* pipes to servers */}
      <path d="M290 100 H420" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cli-a)" />
      <path d="M290 160 H420" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cli-a)" />
      <text x="352" y="92" textAnchor="middle" style={label}>commands</text>

      {/* servers */}
      <rect x="424" y="74" width="180" height="52" rx="10" fill={ACCENT} opacity="0.07" stroke={ACCENT} strokeWidth="1.3" />
      <text x="440" y="98" fontSize="13" fontWeight="700" fill={INK}>Ahnlich DB</text>
      <text x="440" y="116" style={mono} fill={MUTED}>port 1369</text>

      <rect x="424" y="134" width="180" height="52" rx="10" fill={ACCENT} opacity="0.07" stroke={ACCENT} strokeWidth="1.3" />
      <text x="440" y="158" fontSize="13" fontWeight="700" fill={INK}>Ahnlich AI</text>
      <text x="440" y="176" style={mono} fill={MUTED}>port 1370</text>
    </svg>,
    'The CLI is a direct line to the servers. You type short commands and get answers back instantly — perfect for trying things out before you write any code.',
  );
}

/**
 * Client libraries in one picture: same operations, from the language you
 * already use, over gRPC.
 */
export function ClientsFigure(): ReactNode {
  const langs = [
    ['Python', 68],
    ['Node.js', 178],
    ['Go', 288],
    ['Rust', 398],
  ] as const;
  return fig(
    <svg viewBox="0 0 620 260" role="img"
      aria-label="Python, Node, Go and Rust clients all talk to the Ahnlich servers over gRPC"
      style={svgStyle(620)}>
      <Arrow id="cl-a" />

      <text x="20" y="30" style={label}>Your code</text>
      {langs.map(([name, x]) => (
        <g key={name}>
          <rect x={x} y="42" width="128" height="44" rx="10" fill={SURFACE} stroke={LINE} />
          <text x={x + 64} y="70" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>{name}</text>
          <path d={`M${x + 64} 88 V128`} stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cl-a)" />
        </g>
      ))}

      {/* gRPC bus */}
      <rect x="60" y="132" width="480" height="34" rx="8" fill={ACCENT} opacity="0.1" stroke={ACCENT} strokeWidth="1.3" />
      <text x="300" y="154" textAnchor="middle" fontSize="13" fontWeight="700" fill={ACCENT}>gRPC · Protocol Buffers</text>

      <path d="M300 166 V196" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cl-a)" />

      {/* servers */}
      <rect x="150" y="200" width="140" height="44" rx="10" fill={ACCENT} opacity="0.07" stroke={ACCENT} strokeWidth="1.3" />
      <text x="220" y="227" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Ahnlich DB</text>
      <rect x="310" y="200" width="140" height="44" rx="10" fill={ACCENT} opacity="0.07" stroke={ACCENT} strokeWidth="1.3" />
      <text x="380" y="227" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Ahnlich AI</text>
    </svg>,
    'Pick the language you already work in. Every client speaks the same gRPC protocol and exposes the same operations, so the concepts you learn carry across all of them.',
  );
}

/**
 * Installation in one picture: three ways in, all ending at a running server you
 * can PING.
 */
export function InstallFigure(): ReactNode {
  const routes = [
    ['1 · Download binary', 'fastest', 44],
    ['2 · Docker', 'isolated', 108],
    ['3 · Build from source', 'for devs', 172],
  ] as const;
  return fig(
    <svg viewBox="0 0 620 250" role="img"
      aria-label="Three ways to install — download a binary, use Docker, or build from source — all lead to a running server"
      style={svgStyle(620)}>
      <Arrow id="in-a" />

      <text x="20" y="30" style={label}>Choose one</text>
      {routes.map(([title, sub, y]) => (
        <g key={title}>
          <rect x="20" y={y} width="230" height="52" rx="10" fill={SURFACE} stroke={LINE} />
          <text x="36" y={y + 24} fontSize="13" fontWeight="700" fill={INK}>{title}</text>
          <text x="36" y={y + 42} style={mono} fill={MUTED}>{sub}</text>
          <path d={`M250 ${y + 26} H352`} stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#in-a)" />
        </g>
      ))}

      {/* running server */}
      <rect x="356" y="70" width="240" height="112" rx="14" fill={ACCENT} opacity="0.08" stroke={ACCENT} strokeWidth="1.5" />
      <text x="476" y="104" textAnchor="middle" fontSize="14" fontWeight="700" fill={INK}>Running server</text>
      <text x="476" y="132" textAnchor="middle" style={mono} fill={ACCENT}>&gt; PING</text>
      <text x="476" y="152" textAnchor="middle" style={mono} fill={INK}>&lt; PONG ✓</text>
      <text x="476" y="172" textAnchor="middle" fontSize="11" fill={MUTED}>ready for commands</text>
    </svg>,
    'However you install it, you finish the same way: a server running on your machine that answers PING. From there every command works the same.',
  );
}

/**
 * Use cases in one picture: every use case is the same recipe — turn input into a
 * vector, find the nearest matches, optionally filter by metadata.
 */
export function UseCasesFigure(): ReactNode {
  const cases = ['Semantic search', 'Recommendations', 'Cross-modal (text↔image)', 'Clustering'];
  return fig(
    <svg viewBox="0 0 660 250" role="img"
      aria-label="Every use case follows one recipe: embed the input, find nearest vectors, filter by metadata"
      style={svgStyle(660)}>
      <Arrow id="uc-a" />

      {/* input */}
      <rect x="16" y="40" width="150" height="70" rx="12" fill={SURFACE} stroke={LINE} />
      <text x="91" y="68" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Your input</text>
      <text x="91" y="88" textAnchor="middle" style={mono} fill={MUTED}>text · image</text>
      <text x="91" y="104" textAnchor="middle" style={mono} fill={MUTED}>audio · a user</text>

      <path d="M170 75 H234" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#uc-a)" />

      {/* engine */}
      <rect x="238" y="26" width="188" height="98" rx="12" fill={ACCENT} opacity="0.09" stroke={ACCENT} strokeWidth="1.5" />
      <text x="332" y="56" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Ahnlich</text>
      <text x="332" y="80" textAnchor="middle" fontSize="12" fill={MUTED}>find nearest vectors</text>
      <text x="332" y="100" textAnchor="middle" fontSize="12" fill={MUTED}>+ filter by metadata</text>

      <path d="M430 75 H494" stroke={ACCENT} strokeWidth="2" fill="none" markerEnd="url(#uc-a)" />

      {/* results */}
      <rect x="498" y="40" width="150" height="70" rx="12" fill={SURFACE} stroke={LINE} />
      <text x="573" y="68" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>Relevant</text>
      <text x="573" y="88" textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>results</text>
      <text x="573" y="104" textAnchor="middle" style={mono} fill={MUTED}>ranked</text>

      {/* use-case chips */}
      <text x="20" y="164" style={label}>Same recipe powers</text>
      {cases.map((c, i) => {
        const x = 20 + i * 160;
        return (
          <g key={c}>
            <rect x={x} y="176" width="150" height="34" rx="17" fill={ACCENT} opacity="0.07" stroke={ACCENT} strokeWidth="1.2" />
            <text x={x + 75} y="197" textAnchor="middle" fontSize="11.5" fontWeight="600" fill={INK}>{c}</text>
          </g>
        );
      })}
    </svg>,
    'Every use case below is the same three steps: turn your input into a vector, find the nearest stored vectors, and optionally filter by metadata. Only the data and the labels change.',
  );
}
