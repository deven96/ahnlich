import type {ReactNode} from 'react';

/*
 * Lightweight, theme-aware SVG diagrams for the Concepts pages. Colours come
 * from Infima CSS variables so they adapt to light/dark automatically.
 */

const ACCENT = 'var(--ifm-color-primary)';
const INK = 'var(--ifm-color-content)';
const MUTED = 'var(--ifm-color-content-secondary)';
const LINE = 'var(--ifm-toc-border-color)';
const SURFACE = 'var(--ifm-background-surface-color)';

const wrap = {
  margin: '1.5rem 0',
  width: '100%',
} as const;

const label = {
  fontSize: 11,
  letterSpacing: '0.12em',
  textTransform: 'uppercase',
  fill: MUTED,
  fontWeight: 700,
} as const;

/** Anatomy of an entry's value: a set of named metadata fields. */
export function MetadataDiagram(): ReactNode {
  const fields = [
    ['author', '"Asimov"', 'string'],
    ['genre', '"SciFi"', 'string'],
    ['tags', '"epic, space"', 'string'],
    ['cover', '‹image bytes›', 'binary'],
  ];
  const mono = {fontFamily: 'var(--ifm-font-family-monospace)', fontSize: 13} as const;
  return (
    <figure style={wrap}>
      <svg
        viewBox="0 0 560 262"
        role="img"
        aria-label="An entry value is a set of named metadata fields, each with a value and a type"
        style={{width: '100%', height: 'auto', maxWidth: 560, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
        {/* key → value */}
        <rect x="14" y="104" width="150" height="42" rx="8" fill={SURFACE} stroke={LINE} />
        <text x="28" y="130" style={mono} fill={ACCENT}>[0.12, 0.98, …]</text>
        <text x="20" y="96" style={label}>Key</text>
        <path d="M168 125 H210" stroke={LINE} strokeWidth="2" fill="none" markerEnd="url(#cf-md)" />
        <defs>
          <marker id="cf-md" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
            <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
          </marker>
        </defs>

        {/* value container */}
        <rect x="216" y="14" width="330" height="234" rx="14" fill={ACCENT} opacity="0.06" />
        <rect x="216" y="14" width="330" height="234" rx="14" fill="none" stroke={ACCENT} strokeWidth="1.5" />
        <text x="234" y="44" fontSize="14" fontWeight="700" fill={INK}>🧾 Value — metadata</text>

        {/* column headers */}
        <text x="236" y="72" style={label}>Field</text>
        <text x="356" y="72" style={label}>Value</text>
        <text x="492" y="72" style={label}>Type</text>

        {fields.map((f, i) => {
          const y = 92 + i * 38;
          return (
            <g key={i}>
              {i > 0 && <line x1="230" y1={y - 10} x2="532" y2={y - 10} stroke={LINE} strokeWidth="1" opacity="0.6" />}
              <text x="236" y={y + 8} style={mono} fill={INK}>{f[0]}</text>
              <text x="356" y={y + 8} style={mono} fill={ACCENT}>{f[1]}</text>
              <text x="492" y={y + 8} fontSize="12" fill={MUTED}>{f[2]}</text>
            </g>
          );
        })}
      </svg>
      <figcaption style={{fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'}}>
        Metadata is the entry's value — named fields describing it, used to filter and enrich results.
      </figcaption>
    </figure>
  );
}

/** A store as a container of entries, each a key (vector) + value (metadata). */
export function DataModelDiagram(): ReactNode {
  const rows = [
    {key: '[0.12, 0.98, 0.34, 0.55]', val: '{author: Asimov, genre: SciFi}'},
    {key: '[0.44, 0.10, 0.77, 0.02]', val: '{author: Le Guin, genre: SciFi}'},
  ];
  const mono = {fontFamily: 'var(--ifm-font-family-monospace)', fontSize: 12.5} as const;
  return (
    <figure style={wrap}>
      <svg
        viewBox="0 0 620 210"
        role="img"
        aria-label="A store contains entries; each entry is a key vector paired with a value of metadata"
        style={{width: '100%', height: 'auto', maxWidth: 620, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
        {/* store container */}
        <rect x="14" y="14" width="592" height="182" rx="16" fill={ACCENT} opacity="0.06" />
        <rect x="14" y="14" width="592" height="182" rx="16" fill="none" stroke={ACCENT} strokeWidth="1.5" />

        {/* header */}
        <text x="34" y="46" fontSize="15" fontWeight="700" fill={INK}>🗂️ Store — my_store</text>
        <rect x="470" y="28" width="120" height="24" rx="12" fill={SURFACE} stroke={LINE} />
        <text x="530" y="44" textAnchor="middle" fontSize="12" fill={MUTED}>dimension · 4</text>

        {/* column labels */}
        <text x="40" y="76" style={label}>Key — vector</text>
        <text x="326" y="76" style={label}>Value — metadata</text>

        {/* entry rows */}
        {rows.map((r, i) => {
          const y = 88 + i * 50;
          return (
            <g key={i}>
              <rect x="36" y={y} width="272" height="40" rx="8" fill={SURFACE} stroke={LINE} />
              <text x="52" y={y + 25} style={mono} fill={ACCENT}>{r.key}</text>
              <rect x="322" y={y} width="262" height="40" rx="8" fill={SURFACE} stroke={LINE} />
              <text x="338" y={y + 25} style={mono} fill={INK}>{r.val}</text>
            </g>
          );
        })}
      </svg>
      <figcaption style={{fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'}}>
        A store holds entries — each a <strong>key</strong> (vector) paired with a <strong>value</strong> (metadata).
      </figcaption>
    </figure>
  );
}

/** Raw data → embedding model → vector pipeline. */
export function EmbeddingPipeline(): ReactNode {
  return (
    <figure style={wrap}>
      <svg
        viewBox="0 0 760 210"
        role="img"
        aria-label="Raw data is passed through an embedding model to produce a vector"
        style={{width: '100%', height: 'auto', fontFamily: 'inherit'}}>
        {/* connectors */}
        <path d="M175 75 H262" stroke={LINE} strokeWidth="2" fill="none" markerEnd="url(#cf-arrow)" />
        <path d="M175 135 C 215 135, 225 105, 262 105" stroke={LINE} strokeWidth="2" fill="none" markerEnd="url(#cf-arrow)" />
        <path d="M452 105 H520" stroke={LINE} strokeWidth="2" fill="none" markerEnd="url(#cf-arrow)" />
        <defs>
          <marker id="cf-arrow" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
            <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
          </marker>
        </defs>

        {/* raw inputs */}
        <g>
          <rect x="20" y="50" width="155" height="50" rx="10" fill={SURFACE} stroke={LINE} />
          <text x="97" y="80" textAnchor="middle" fontSize="14" fill={INK}>“a fast red car”</text>
          <rect x="20" y="112" width="155" height="46" rx="10" fill={SURFACE} stroke={LINE} />
          <text x="97" y="140" textAnchor="middle" fontSize="14" fill={INK}>🖼️  photo.jpg</text>
        </g>

        {/* model */}
        <rect x="262" y="70" width="190" height="70" rx="14" fill={ACCENT} opacity="0.12" />
        <rect x="262" y="70" width="190" height="70" rx="14" fill="none" stroke={ACCENT} strokeWidth="1.5" />
        <text x="357" y="100" textAnchor="middle" fontSize="15" fontWeight="700" fill={INK}>Embedding</text>
        <text x="357" y="120" textAnchor="middle" fontSize="15" fontWeight="700" fill={INK}>model</text>

        {/* vector */}
        <rect x="520" y="78" width="220" height="54" rx="12" fill={SURFACE} stroke={LINE} />
        <text x="630" y="111" textAnchor="middle" fontSize="14" fill={ACCENT} style={{fontFamily: 'var(--ifm-font-family-monospace)'}}>[0.91, 0.12, 0.44, …]</text>

        {/* captions */}
        <text x="97" y="185" textAnchor="middle" style={label}>Raw data</text>
        <text x="357" y="185" textAnchor="middle" style={label}>Model</text>
        <text x="630" y="185" textAnchor="middle" style={label}>Vector</text>
      </svg>
      <figcaption style={{fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'}}>
        An embedding model turns text or images into a vector.
      </figcaption>
    </figure>
  );
}

/** 2-D projection: similar meanings cluster; unrelated ones sit far apart. */
export function SimilaritySpace(): ReactNode {
  return (
    <figure style={wrap}>
      <svg
        viewBox="0 0 480 320"
        role="img"
        aria-label="Similar items are close together in vector space; unrelated items are far apart"
        style={{width: '100%', height: 'auto', maxWidth: 460, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
        {/* axes */}
        <line x1="55" y1="275" x2="440" y2="275" stroke={LINE} strokeWidth="1.5" />
        <line x1="55" y1="275" x2="55" y2="35" stroke={LINE} strokeWidth="1.5" />

        {/* cluster of similar items */}
        <ellipse cx="335" cy="105" rx="72" ry="52" fill={ACCENT} opacity="0.10" />
        <circle cx="315" cy="90" r="6" fill={ACCENT} />
        <text x="326" y="86" fontSize="13" fill={INK}>“fast red car”</text>
        <circle cx="352" cy="120" r="6" fill={ACCENT} />
        <text x="363" y="124" fontSize="13" fill={INK}>“crimson automobile”</text>

        {/* far, unrelated item */}
        <circle cx="140" cy="215" r="6" fill={MUTED} />
        <text x="152" y="219" fontSize="13" fill={MUTED}>“bowl of soup”</text>

        <text x="240" y="305" textAnchor="middle" style={label}>vector space (2-D projection)</text>
      </svg>
      <figcaption style={{fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'}}>
        Similar meanings land close together; unrelated ones fall far apart.
      </figcaption>
    </figure>
  );
}

const cap = {fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'} as const;
const fig = (svg: ReactNode, caption: ReactNode): ReactNode => (
  <figure style={wrap}>
    {svg}
    <figcaption style={cap}>{caption}</figcaption>
  </figure>
);

/** Full scan vs. an index that narrows the search to a few candidates. */
export function IndexDiagram(): ReactNode {
  const cols = [190, 235, 280];
  const rcols = [560, 605, 650];
  const ys = [95, 150, 205];
  return fig(
    <svg viewBox="0 0 750 290" role="img"
      aria-label="Without an index every vector is compared; with an index only a few candidates are checked"
      style={{width: '100%', height: 'auto', fontFamily: 'inherit'}}>
      {/* left: full scan */}
      <rect x="12" y="34" width="352" height="228" rx="14" fill="none" stroke={LINE} />
      <text x="30" y="60" fontSize="14" fontWeight="700" fill={INK}>Without an index</text>
      {cols.map((x) => ys.map((y) => (
        <line key={`l${x}-${y}`} x1="58" y1="150" x2={x} y2={y} stroke={LINE} strokeWidth="1" />
      )))}
      <circle cx="58" cy="150" r="8" fill={ACCENT} />
      {cols.map((x) => ys.map((y) => <circle key={`d${x}-${y}`} cx={x} cy={y} r="6" fill={MUTED} />))}
      <text x="188" y="248" textAnchor="middle" style={label}>every vector compared</text>

      {/* right: indexed */}
      <rect x="386" y="34" width="352" height="228" rx="14" fill="none" stroke={LINE} />
      <text x="404" y="60" fontSize="14" fontWeight="700" fill={INK}>With an index</text>
      <ellipse cx="605" cy="150" rx="34" ry="90" fill={ACCENT} opacity="0.10" />
      {ys.map((y) => <line key={`rl${y}`} x1="432" y1="150" x2="605" y2={y} stroke={ACCENT} strokeWidth="1.5" />)}
      <circle cx="432" cy="150" r="8" fill={ACCENT} />
      {rcols.map((x) => ys.map((y) => (
        <circle key={`rd${x}-${y}`} cx={x} cy={y} r="6" fill={x === 605 ? ACCENT : MUTED} opacity={x === 605 ? 1 : 0.35} />
      )))}
      <text x="562" y="248" textAnchor="middle" style={label}>only candidates checked</text>
    </svg>,
    'An index skips most of the store instead of scanning every vector.',
  );
}

/** Exact scan: the query is compared against every stored vector. */
export function ExactScanFigure(): ReactNode {
  const dots = [
    [190, 70], [270, 60], [350, 95], [420, 70],
    [200, 150], [300, 140], [400, 165],
    [190, 235], [280, 240], [370, 230], [430, 210],
  ];
  return fig(
    <svg viewBox="0 0 470 300" role="img"
      aria-label="The query is compared against every stored vector"
      style={{width: '100%', height: 'auto', maxWidth: 470, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
      {dots.map(([x, y], i) => (
        <line key={i} x1="70" y1="150" x2={x} y2={y} stroke={LINE} strokeWidth="1" />
      ))}
      <circle cx="70" cy="150" r="9" fill={ACCENT} />
      <text x="70" y="128" textAnchor="middle" fontSize="12" fill={INK}>query</text>
      {dots.map(([x, y], i) => <circle key={`d${i}`} cx={x} cy={y} r="6" fill={MUTED} />)}
      <text x="290" y="288" textAnchor="middle" style={label}>every vector measured</text>
    </svg>,
    'No shortcuts: the query is compared with all N vectors — always exact, but the work grows with N.',
  );
}

/** HNSW: a layered graph you traverse to hop toward the nearest neighbour. */
export function HnswFigure(): ReactNode {
  const top = [[80, 80], [210, 60], [330, 90], [440, 70]];
  const bot = [[60, 210], [120, 230], [180, 205], [250, 235], [310, 205], [370, 232], [440, 210]];
  return fig(
    <svg viewBox="0 0 500 280" role="img"
      aria-label="Start in a sparse top layer and hop toward the target, then refine in a denser bottom layer"
      style={{width: '100%', height: 'auto', maxWidth: 500, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
      <text x="12" y="40" style={label}>Layer 1 · sparse</text>
      <text x="12" y="175" style={label}>Layer 0 · dense</text>

      {/* sparse edges */}
      <path d="M80 80 H210 M210 60 H330 M330 90 H440" stroke={LINE} strokeWidth="1.2" />
      {/* dense edges */}
      <path d="M60 210 H120 M120 230 H180 M180 205 H250 M250 235 H310 M310 205 H370 M370 232 H440" stroke={LINE} strokeWidth="1.2" />
      {/* cross-layer links */}
      {[[80, 80, 60, 210], [210, 60, 180, 205], [330, 90, 310, 205], [440, 70, 440, 210]].map(([x1, y1, x2, y2], i) => (
        <line key={i} x1={x1} y1={y1} x2={x2} y2={y2} stroke={LINE} strokeWidth="1" strokeDasharray="3 3" />
      ))}

      {/* traversal path (accent) */}
      <path d="M80 80 H330" stroke={ACCENT} strokeWidth="2.5" />
      <path d="M330 90 L310 205" stroke={ACCENT} strokeWidth="2.5" strokeDasharray="3 3" />
      <path d="M310 205 H370" stroke={ACCENT} strokeWidth="2.5" />

      {top.map(([x, y], i) => <circle key={`t${i}`} cx={x} cy={y} r="6" fill={MUTED} />)}
      {bot.map(([x, y], i) => <circle key={`b${i}`} cx={x} cy={y} r="6" fill={MUTED} />)}
      <circle cx="80" cy="80" r="7" fill={ACCENT} />
      <text x="80" y="62" textAnchor="middle" fontSize="12" fill={INK}>entry</text>
      <circle cx="370" cy="232" r="7" fill={ACCENT} />
      <text x="392" y="236" fontSize="12" fill={INK}>target</text>
    </svg>,
    'Start sparse and hop toward the answer, then drop into a denser layer to refine — a few hops instead of a full scan.',
  );
}

/** Schemas as namespaces; the same store name can live in each, isolated. */
export function SchemaDiagram(): ReactNode {
  const box = (x: number, name: string, stores: string[]) => (
    <g>
      <rect x={x} y="30" width="270" height="176" rx="14" fill={ACCENT} opacity="0.05" />
      <rect x={x} y="30" width="270" height="176" rx="14" fill="none" stroke={LINE} />
      <text x={x + 20} y="58" fontSize="14" fontWeight="700" fill={INK}>📁 {name}</text>
      {stores.map((s, i) => {
        const hl = s === 'my_store';
        return (
          <g key={s}>
            <rect x={x + 20} y={78 + i * 52} width="230" height="40" rx="9" fill={SURFACE}
              stroke={hl ? ACCENT : LINE} strokeWidth={hl ? 1.5 : 1} />
            <text x={x + 38} y={103 + i * 52} fontSize="13" fill={INK}
              style={{fontFamily: 'var(--ifm-font-family-monospace)'}}>🗂️ {s}</text>
          </g>
        );
      })}
    </g>
  );
  return fig(
    <svg viewBox="0 0 620 236" role="img"
      aria-label="Two schemas each contain stores; the same store name exists in both, isolated"
      style={{width: '100%', height: 'auto', maxWidth: 620, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
      {box(20, 'public', ['my_store', 'users'])}
      {box(330, 'analytics', ['my_store', 'events'])}
    </svg>,
    <>The same store name — <code>my_store</code> — lives in each schema, fully isolated.</>,
  );
}

/** DB store (you send a vector) vs AI store (send text/image, server embeds it). */
export function AIStoreDiagram(): ReactNode {
  const chip = (x: number, y: number, w: number, text: string, accent = false) => (
    <g>
      <rect x={x} y={y} width={w} height="40" rx="9" fill={accent ? ACCENT : SURFACE}
        opacity={accent ? 0.12 : 1} stroke={accent ? ACCENT : LINE} strokeWidth={accent ? 1.5 : 1} />
      <text x={x + w / 2} y={y + 25} textAnchor="middle" fontSize="13" fill={INK}>{text}</text>
    </g>
  );
  const arrow = (x1: number, x2: number, y: number) => (
    <path d={`M${x1} ${y} H${x2}`} stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cf-ai)" />
  );
  return fig(
    <svg viewBox="0 0 740 250" role="img"
      aria-label="A DB store takes a vector directly; an AI store takes text or an image and embeds it"
      style={{width: '100%', height: 'auto', fontFamily: 'inherit'}}>
      <defs>
        <marker id="cf-ai" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
          <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
        </marker>
      </defs>
      {/* shared store */}
      <rect x="590" y="55" width="130" height="130" rx="14" fill={ACCENT} opacity="0.10" />
      <rect x="590" y="55" width="130" height="130" rx="14" fill="none" stroke={ACCENT} strokeWidth="1.5" />
      <text x="655" y="125" textAnchor="middle" fontSize="15" fontWeight="700" fill={INK}>Store</text>

      {/* DB lane */}
      <text x="30" y="60" style={label}>Vector DB store · :1369</text>
      {chip(30, 72, 180, '[0.91, 0.12, …]  vector')}
      {arrow(214, 588, 92)}

      {/* AI lane */}
      <text x="30" y="150" style={label}>AI store · :1370</text>
      {chip(30, 162, 150, 'text / image')}
      {arrow(184, 246, 182)}
      {chip(250, 162, 150, 'embedding model', true)}
      {arrow(404, 588, 182)}
    </svg>,
    'DB stores take a vector directly; AI stores take text or an image and embed it for you (index model = stored data, query model = searches).',
  );
}

/** A predicate selecting the metadata rows that match. */
export function MetadataFilterDiagram(): ReactNode {
  const rows: [string, string, boolean][] = [
    ['Dune', 'SciFi', true],
    ['Emma', 'Romance', false],
    ['Foundation', 'SciFi', true],
    ['Hamlet', 'Drama', false],
  ];
  return fig(
    <svg viewBox="0 0 560 300" role="img"
      aria-label="A predicate genre equals SciFi highlights the matching metadata rows"
      style={{width: '100%', height: 'auto', maxWidth: 560, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
      {/* predicate chip */}
      <rect x="190" y="14" width="180" height="38" rx="19" fill={ACCENT} opacity="0.14" />
      <rect x="190" y="14" width="180" height="38" rx="19" fill="none" stroke={ACCENT} strokeWidth="1.5" />
      <text x="280" y="39" textAnchor="middle" fontSize="14" fill={INK}
        style={{fontFamily: 'var(--ifm-font-family-monospace)'}}>genre = SciFi</text>
      <path d="M280 52 V72" stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cf-mf)" />
      <defs>
        <marker id="cf-mf" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
          <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
        </marker>
      </defs>
      {/* header */}
      <text x="60" y="96" style={label}>Title</text>
      <text x="300" y="96" style={label}>genre</text>
      {/* rows */}
      {rows.map(([title, genre, match], i) => {
        const y = 108 + i * 44;
        return (
          <g key={title} opacity={match ? 1 : 0.4}>
            <rect x="48" y={y} width="464" height="36" rx="8"
              fill={match ? ACCENT : SURFACE} opacity={match ? 0.12 : 1} stroke={match ? ACCENT : LINE} />
            <text x="64" y={y + 23} fontSize="13" fill={INK}>{title}</text>
            <text x="300" y={y + 23} fontSize="13" fill={INK}>{genre}</text>
            {match && <text x="490" y={y + 24} fontSize="15" fill={ACCENT} textAnchor="middle">✓</text>}
          </g>
        );
      })}
    </svg>,
    'A predicate is a condition over metadata — it selects the entries whose fields match.',
  );
}

/** End-to-end map: ingest on top, query on the bottom, meeting at the store/index. */
export function BigPictureDiagram(): ReactNode {
  const node = (x: number, y: number, w: number, title: string, sub?: string, accent = false) => (
    <g>
      <rect x={x} y={y} width={w} height="52" rx="12" fill={accent ? ACCENT : SURFACE}
        opacity={accent ? 0.1 : 1} stroke={accent ? ACCENT : LINE} strokeWidth={accent ? 1.5 : 1} />
      <text x={x + w / 2} y={sub ? y + 24 : y + 31} textAnchor="middle" fontSize="13" fontWeight="700" fill={INK}>{title}</text>
      {sub && <text x={x + w / 2} y={y + 40} textAnchor="middle" fontSize="11" fill={MUTED}>{sub}</text>}
    </g>
  );
  const arr = (x1: number, x2: number, y: number) => (
    <path d={`M${x1} ${y} H${x2}`} stroke={MUTED} strokeWidth="2" fill="none" markerEnd="url(#cf-bp)" />
  );
  return fig(
    <svg viewBox="0 0 740 250" role="img"
      aria-label="Ingest flow: data, embed, store with metadata, index. Query flow: query, embed, similarity search, ranked results."
      style={{width: '100%', height: 'auto', fontFamily: 'inherit'}}>
      <defs>
        <marker id="cf-bp" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
          <path d="M0 0 L6 3 L0 6 Z" fill={MUTED} />
        </marker>
      </defs>
      <text x="20" y="30" style={label}>Ingest</text>
      {node(20, 40, 120, 'Your data', 'text · image · vectors')}
      {arr(140, 168, 66)}
      {node(168, 40, 110, 'Embed', 'AI stores only')}
      {arr(278, 306, 66)}
      {node(306, 40, 150, 'Store', 'key + metadata', true)}
      {arr(456, 484, 66)}
      {node(484, 40, 120, 'Index')}

      {/* link store/index to query lane */}
      <path d="M544 92 V150" stroke={MUTED} strokeWidth="2" strokeDasharray="4 4" fill="none" markerEnd="url(#cf-bp)" />

      <text x="20" y="150" style={label}>Query</text>
      {node(20, 160, 120, 'Your query', 'text · image · vector')}
      {arr(140, 168, 186)}
      {node(168, 160, 110, 'Embed', 'AI stores only')}
      {arr(278, 306, 186)}
      {node(306, 160, 150, 'Similarity search', undefined, true)}
      {arr(456, 484, 186)}
      {node(484, 160, 150, 'Ranked results', '+ metadata')}
    </svg>,
    'Ingest your data once (top); query it as often as you like (bottom). The index makes search fast at scale.',
  );
}

/** Cosine (angle) vs Euclidean (distance) between two vectors from the origin. */
export function MetricsDiagram(): ReactNode {
  return (
    <figure style={wrap}>
      <svg
        viewBox="0 0 460 250"
        role="img"
        aria-label="Cosine similarity compares the angle between vectors; Euclidean distance compares the gap between their tips"
        style={{width: '100%', height: 'auto', maxWidth: 460, display: 'block', margin: '0 auto', fontFamily: 'inherit'}}>
        {/* axes */}
        <line x1="60" y1="210" x2="430" y2="210" stroke={LINE} strokeWidth="1.5" />
        <line x1="60" y1="210" x2="60" y2="30" stroke={LINE} strokeWidth="1.5" />

        {/* two vectors from origin */}
        <line x1="60" y1="210" x2="330" y2="70" stroke={ACCENT} strokeWidth="2.5" markerEnd="url(#cf-a2)" />
        <line x1="60" y1="210" x2="360" y2="150" stroke={ACCENT} strokeWidth="2.5" markerEnd="url(#cf-a2)" />
        <defs>
          <marker id="cf-a2" markerWidth="9" markerHeight="9" refX="6" refY="3" orient="auto">
            <path d="M0 0 L6 3 L0 6 Z" fill={ACCENT} />
          </marker>
        </defs>

        {/* angle arc (cosine) near origin */}
        <path d="M110 184 A 55 55 0 0 0 118 154" fill="none" stroke={INK} strokeWidth="1.5" />
        <text x="128" y="182" fontSize="13" fill={INK}>θ (cosine)</text>

        {/* distance between tips (euclidean) */}
        <line x1="330" y1="70" x2="360" y2="150" stroke={MUTED} strokeWidth="1.5" strokeDasharray="4 4" />
        <text x="352" y="105" fontSize="13" fill={MUTED}>distance</text>

        <text x="336" y="60" fontSize="13" fill={INK}>A</text>
        <text x="368" y="150" fontSize="13" fill={INK}>B</text>
      </svg>
      <figcaption style={{fontSize: '0.85rem', color: MUTED, textAlign: 'center', marginTop: '0.25rem'}}>
        Cosine compares the <strong>angle</strong> θ; Euclidean compares the <strong>distance</strong> between the tips.
      </figcaption>
    </figure>
  );
}
