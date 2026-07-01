import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';

type Highlight = {
  icon: ReactNode;
  title: string;
  description: string;
  href: string;
};

/* Lucide-style line icons, size controlled by the wrapping element */
const iconProps = {
  viewBox: '0 0 24 24',
  fill: 'none',
  stroke: 'currentColor',
  strokeWidth: 1.5,
  strokeLinecap: 'round' as const,
  strokeLinejoin: 'round' as const,
  className: 'h-full w-full',
};

const BoltIcon = () => (
  <svg {...iconProps}>
    <path d="M13 2 3 14h9l-1 8 10-12h-9l1-8z" />
  </svg>
);
const BrainIcon = () => (
  <svg {...iconProps}>
    <path d="M12 5a3 3 0 1 0-5.997.125 4 4 0 0 0-2.526 5.77 4 4 0 0 0 .556 6.588A4 4 0 1 0 12 18Z" />
    <path d="M12 5a3 3 0 1 1 5.997.125 4 4 0 0 1 2.526 5.77 4 4 0 0 1-.556 6.588A4 4 0 1 1 12 18Z" />
    <path d="M15 13a4.5 4.5 0 0 1-3-4 4.5 4.5 0 0 1-3 4" />
  </svg>
);
const SearchIcon = () => (
  <svg {...iconProps}>
    <circle cx="11" cy="11" r="8" />
    <path d="m21 21-4.3-4.3" />
  </svg>
);
const PuzzleIcon = () => (
  <svg {...iconProps}>
    <path d="M15.39 4.39a1 1 0 0 0 1.68-.474 2.5 2.5 0 1 1 3.014 3.015 1 1 0 0 0-.474 1.68l1.683 1.682a2.414 2.414 0 0 1 0 3.414L20.61 16.39a1 1 0 0 1-1.68-.474 2.5 2.5 0 1 0-3.014 3.015 1 1 0 0 1 .474 1.68l-1.683 1.682a2.414 2.414 0 0 1-3.414 0L9.61 20.61a1 1 0 0 0-1.68.474 2.5 2.5 0 1 1-3.014-3.015 1 1 0 0 0 .474-1.68l-1.683-1.682a2.414 2.414 0 0 1 0-3.414L5.39 9.61a1 1 0 0 1 1.68.474 2.5 2.5 0 1 0 3.014-3.015 1 1 0 0 1-.474-1.68l1.683-1.682a2.414 2.414 0 0 1 3.414 0z" />
  </svg>
);
const DatabaseIcon = () => (
  <svg {...iconProps}>
    <ellipse cx="12" cy="5" rx="9" ry="3" />
    <path d="M3 5V19A9 3 0 0 0 21 19V5" />
    <path d="M3 12A9 3 0 0 0 21 12" />
  </svg>
);
const GlobeIcon = () => (
  <svg {...iconProps}>
    <circle cx="12" cy="12" r="10" />
    <path d="M12 2a14.5 14.5 0 0 0 0 20 14.5 14.5 0 0 0 0-20" />
    <path d="M2 12h20" />
  </svg>
);

const HIGHLIGHTS: Highlight[] = [
  {
    icon: <BoltIcon />,
    title: 'Blazing fast',
    description:
      'A lock-free, in-memory core built in Rust with SIMD-accelerated similarity search for low-latency retrieval at scale.',
    href: '/docs/components/ahnlich-db',
  },
  {
    icon: <BrainIcon />,
    title: 'AI built in',
    description:
      'A first-class AI proxy runs ONNX embedding models for you. Send raw text or images and let Ahnlich handle the vectors.',
    href: '/docs/components/ahnlich-ai',
  },
  {
    icon: <SearchIcon />,
    title: 'Hybrid search',
    description:
      'Combine semantic vector search with structured metadata predicates to filter results with surgical precision.',
    href: '/docs/components/predicates',
  },
  {
    icon: <PuzzleIcon />,
    title: 'Pluggable models & metrics',
    description:
      'Bring your own embedding models and choose the similarity metric (cosine, Euclidean, dot product) per store.',
    href: '/docs/components/ahnlich-ai/advanced',
  },
  {
    icon: <DatabaseIcon />,
    title: 'Durable by design',
    description:
      'Snapshot persistence with deterministic hashing keeps your data consistent across restarts and upgrades.',
    href: '/docs/components/persistence-in-ahnlich',
  },
  {
    icon: <GlobeIcon />,
    title: 'Polyglot clients',
    description:
      'gRPC and Protocol Buffers power native SDKs for Rust, Python, and Node.js so you can build in any stack.',
    href: '/docs/client-libraries',
  },
];

function HighlightCard({icon, title, description, href}: Highlight) {
  return (
    <Link
      to={href}
      className="group relative flex min-h-[16rem] flex-col overflow-hidden rounded-2xl border border-solid border-black/[0.06] bg-white p-8 text-left text-inherit no-underline shadow-sm transition-all duration-300 hover:-translate-y-1 hover:border-primary/30 hover:text-inherit hover:no-underline hover:shadow-xl dark:border-white/10 dark:bg-white/[0.03] dark:hover:border-primary/40">
      {/* oversized icon bleeding off the top-right edge as a translucent overlay */}
      <div
        aria-hidden
        className="pointer-events-none absolute -right-6 -top-6 h-40 w-40 rotate-12 text-primary/[0.07] transition-all duration-500 group-hover:-right-4 group-hover:rotate-6 group-hover:text-primary/[0.14] dark:text-white/[0.05] dark:group-hover:text-primary/20">
        {icon}
      </div>
      {/* hover glow */}
      <div className="pointer-events-none absolute -right-10 -top-10 h-32 w-32 rounded-full bg-primary/10 opacity-0 blur-2xl transition-opacity duration-300 group-hover:opacity-100" />

      <div className="relative flex h-full flex-col">
        {/* small solid icon chip */}
        <div className="mb-6 inline-flex h-11 w-11 items-center justify-center rounded-xl bg-gradient-to-br from-primary to-secondary p-2.5 text-white shadow-lg shadow-primary/20 transition-transform duration-300 group-hover:scale-110">
          {icon}
        </div>
        <h3 className="mb-2 text-xl font-semibold">{title}</h3>
        <p className="m-0 text-[0.95rem] leading-relaxed opacity-70">
          {description}
        </p>
        {/* read more: minimal arrow that shoots out on hover */}
        <span className="mt-auto inline-flex items-center gap-2 pt-6 text-sm font-semibold text-primary">
          Read more
          <span className="inline-flex items-center">
            <span className="h-[1.5px] w-0 rounded-full bg-primary transition-all duration-300 ease-out group-hover:w-4" />
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2.5}
              strokeLinecap="round"
              strokeLinejoin="round"
              className="-ml-1 h-4 w-4">
              <path d="m9 6 6 6-6 6" />
            </svg>
          </span>
        </span>
      </div>
    </Link>
  );
}

export default function Highlights(): ReactNode {
  return (
    <section className="bg-white py-24 dark:bg-[#242526]">
      <div className="container">
        <div className="mx-auto mb-16 max-w-3xl text-center">
          <h2 className="text-3xl font-bold leading-tight md:text-5xl">
            Everything you need to ship{' '}
            <span className="bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
              semantic search
            </span>
          </h2>
          <p className="mx-auto mt-5 max-w-2xl text-lg leading-relaxed opacity-70">
            One fast, open-source engine that handles it all: embed, index,
            filter, and retrieve, so you can focus on your product, not your
            infrastructure.
          </p>
        </div>
        <div className="grid grid-cols-1 gap-6 md:grid-cols-2 lg:grid-cols-3">
          {HIGHLIGHTS.map((h, idx) => (
            <HighlightCard key={idx} {...h} />
          ))}
        </div>
      </div>
    </section>
  );
}
