import {useState, type ReactNode} from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

import Highlights from '@site/src/components/Highlights';
import PythonApi from '@site/src/components/PythonApi';

import Logo from '@site/static/img/logo.png';

import {ActionLinks} from '../components/buttons';
import {DownloadIcon, GithubIcon, RocketIcon} from '../components/icons';
import AudioPlayer from '../components/AudioPlayer';

const SDKS = [
  {label: 'Rust', command: 'cargo add ahnlich_client_rs'},
  {label: 'Python', command: 'pip install ahnlich-client-py'},
  {label: 'Node', command: 'npm install ahnlich-client-node'},
  {label: 'Go', command: 'go get github.com/deven96/ahnlich/sdk/ahnlich-client-go'},
];

function InstallSwitcher() {
  const [active, setActive] = useState(0);
  const [copied, setCopied] = useState(false);
  const command = SDKS[active].command;

  const onCopy = () => {
    navigator.clipboard?.writeText(command).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    });
  };

  return (
    <div className="w-full max-w-md overflow-hidden rounded-xl border border-solid border-white/15 bg-white/5 backdrop-blur">
      {/* SDK tabs */}
      <div className="flex border-b border-solid border-white/10">
        {SDKS.map((sdk, idx) => (
          <button
            key={sdk.label}
            onClick={() => {
              setActive(idx);
              setCopied(false);
            }}
            className={`flex-1 bg-transparent px-3 py-2 text-xs font-semibold transition-colors ${
              idx === active
                ? 'bg-white/10 text-white'
                : 'text-white/50 hover:text-white/80'
            }`}>
            {sdk.label}
          </button>
        ))}
      </div>
      {/* command row */}
      <button
        onClick={onCopy}
        title="Copy to clipboard"
        className="group flex w-full items-center gap-3 bg-transparent px-4 py-3 text-left font-mono text-sm text-white/90 transition-colors hover:bg-white/[0.04]">
        <span className="select-none text-secondary">$</span>
        <span className="flex-1 truncate">{command}</span>
        <span className="text-white/50 transition-colors group-hover:text-white">
          {copied ? (
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2.5}
              strokeLinecap="round"
              strokeLinejoin="round"
              className="h-4 w-4 text-secondary">
              <path d="M20 6 9 17l-5-5" />
            </svg>
          ) : (
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2}
              strokeLinecap="round"
              strokeLinejoin="round"
              className="h-4 w-4">
              <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
              <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
            </svg>
          )}
        </span>
      </button>
    </div>
  );
}

function HomepageHeader() {
  return (
    <header className="relative overflow-hidden bg-[#08161d] text-white">
      {/* animated background layers */}
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-30 [mask-image:radial-gradient(ellipse_at_center,black,transparent_75%)]" />
      <div className="pointer-events-none absolute left-1/2 top-[-10%] h-[38rem] w-[38rem] -translate-x-1/2 rounded-full bg-primary/20 blur-[120px]" />
      <div className="ahn-blob pointer-events-none absolute -left-32 top-10 h-96 w-96 rounded-full bg-secondary/25 blur-3xl" />
      <div
        className="ahn-blob pointer-events-none absolute -right-24 top-32 h-96 w-96 rounded-full bg-primary/25 blur-3xl"
        style={{animationDelay: '-6s'}}
      />
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-[#08161d] to-transparent" />

      <div className="container relative z-10 flex flex-col items-center py-24 text-center md:py-32">
        <div className="ahn-fade-up mb-8 flex items-center gap-3">
          <img
            src={Logo}
            alt="Ahnlich logo"
            className="h-12 w-12 drop-shadow-[0_0_25px_rgba(9,181,202,0.5)]"
          />
          <span className="text-xl font-bold tracking-tight">Ahnlich</span>
        </div>

        <Link
          href="https://github.com/deven96/ahnlich"
          className="ahn-fade-up mb-8 inline-flex items-center gap-2 rounded-full border border-solid border-white/15 bg-white/5 px-4 py-1.5 text-sm text-white/90 no-underline backdrop-blur transition-colors hover:border-secondary/50 hover:text-white"
          style={{animationDelay: '0.05s'}}>
          <span className="relative flex h-2 w-2">
            <span className="absolute inline-flex h-2 w-2 animate-ping rounded-full bg-secondary opacity-75" />
            <span className="relative inline-flex h-2 w-2 rounded-full bg-secondary" />
          </span>
          Open source, Rust-native, AI-powered
          <span className="opacity-50">→</span>
        </Link>

        <h1
          className="ahn-fade-up max-w-4xl text-4xl font-extrabold leading-[1.08] tracking-tight md:text-6xl lg:text-7xl"
          style={{animationDelay: '0.1s'}}>
          The vector database that{' '}
          <span className="bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
            gets out of your way
          </span>
        </h1>

        <p
          className="ahn-fade-up mt-6 max-w-2xl text-lg text-white/70 md:text-xl"
          style={{animationDelay: '0.16s'}}>
          A high-performance, AI-powered in-memory vector database for semantic
          search. Lightning fast, developer friendly, and open source.
        </p>

        <div
          className="ahn-fade-up mt-10 flex flex-col items-center gap-4 sm:flex-row"
          style={{animationDelay: '0.22s'}}>
          <ActionLinks href="/docs/getting-started" icon={<RocketIcon />}>
            Get Started
          </ActionLinks>
          <ActionLinks
            href="https://github.com/deven96/ahnlich"
            icon={<GithubIcon />}>
            View on GitHub
          </ActionLinks>
        </div>

        <div
          className="ahn-fade-up mt-10 flex w-full flex-col items-center gap-3"
          style={{animationDelay: '0.28s'}}>
          <InstallSwitcher />
          <div className="flex items-center gap-2 text-sm text-white/40">
            <span className="font-mono">/ˈɛːnlɪç/</span>
            <span className="text-white/70">
              <AudioPlayer src="/audio/aehnlich.mp3" />
            </span>
          </div>
        </div>
      </div>

      {/* scroll cue */}
      <div className="pointer-events-none absolute bottom-6 left-1/2 z-10 -translate-x-1/2 text-white/30">
        <svg
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          strokeWidth={2}
          strokeLinecap="round"
          strokeLinejoin="round"
          className="h-5 w-5 animate-bounce">
          <path d="m6 9 6 6 6-6" />
        </svg>
      </div>
    </header>
  );
}

const QUICKSTART = `# Run the Ahnlich DB server
cargo install ahnlich_db
ahnlich_db run --port 1369

# ...or spin up the AI proxy for automatic embeddings
cargo install ahnlich_ai
ahnlich_ai run --port 1370`;

function Quickstart() {
  return (
    <section className="bg-slate-100 py-24 dark:bg-[#1b1c1d]">
      <div className="container grid items-center gap-12 lg:grid-cols-2">
        <div className="text-left">
          <span className="mb-3 inline-block rounded-full bg-primary/10 px-4 py-1 text-sm font-semibold text-primary">
            Quickstart
          </span>
          <h2 className="text-3xl font-bold md:text-4xl">
            Up and running in seconds
          </h2>
          <p className="mt-4 text-lg opacity-80">
            Install a single binary and start indexing. No cluster to babysit, no
            heavyweight dependencies. Just a fast vector store ready for your
            embeddings.
          </p>
          <div className="mt-8 flex flex-wrap items-center gap-4">
            <ActionLinks href="/docs/getting-started" icon={<RocketIcon />}>
              Read the Quickstart
            </ActionLinks>
            <Link
              to="/docs/client-libraries"
              className="text-lg font-medium text-primary hover:text-primary/75">
              Explore client SDKs →
            </Link>
          </div>
        </div>
        <div className="overflow-hidden rounded-2xl border border-solid border-grey-1/40 shadow-2xl dark:border-white/10">
          <div className="flex items-center gap-2 bg-[#0b1f28] px-4 py-3">
            <span className="h-3 w-3 rounded-full bg-[#ff5f56]" />
            <span className="h-3 w-3 rounded-full bg-[#ffbd2e]" />
            <span className="h-3 w-3 rounded-full bg-[#27c93f]" />
            <span className="ml-3 text-xs font-medium text-white/50">
              terminal
            </span>
          </div>
          <pre className="m-0 overflow-x-auto bg-[#0b1f28] p-6 text-sm leading-relaxed text-slate-100">
            <code>{QUICKSTART}</code>
          </pre>
        </div>
      </div>
    </section>
  );
}

const STATS = [
  {value: 'Sub-ms', label: 'Similarity search latency'},
  {value: '3+', label: 'Native client SDKs'},
  {value: 'ONNX', label: 'Built-in AI embeddings'},
  {value: '100%', label: 'Open source'},
];

function Stats() {
  return (
    <section className="relative overflow-hidden bg-[#0a1a22] py-16 text-white">
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-20" />
      <div className="pointer-events-none absolute left-1/2 top-0 h-40 w-[40rem] -translate-x-1/2 rounded-full bg-primary/20 blur-3xl" />
      <div className="container relative">
        <div className="grid grid-cols-2 gap-px overflow-hidden rounded-2xl border border-solid border-white/10 bg-white/10 md:grid-cols-4">
          {STATS.map((s) => (
            <div
              key={s.label}
              className="group flex flex-col items-center gap-2 bg-[#0c1e28] px-4 py-10 text-center transition-colors duration-300 hover:bg-white/[0.04]">
              <span className="bg-gradient-to-r from-primary to-secondary bg-clip-text text-4xl font-extrabold text-transparent transition-transform duration-300 group-hover:scale-110 md:text-5xl">
                {s.value}
              </span>
              <span className="text-sm font-medium uppercase tracking-wider text-white/50">
                {s.label}
              </span>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}

function CallToAction() {
  return (
    <section className="relative overflow-hidden bg-[#0a1a22] py-24 text-center text-white">
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-30" />
      <div className="pointer-events-none absolute left-1/2 top-0 h-72 w-[36rem] -translate-x-1/2 rounded-full bg-secondary/20 blur-3xl" />
      <div className="container relative z-10">
        <h2 className="text-3xl font-bold md:text-4xl">
          Start building smarter search today
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-lg text-white/80">
          Grab a release for Mac or Linux, or build from source. Ahnlich is free
          and open source with an active community.
        </p>
        <div className="mt-10 flex flex-col items-center justify-center gap-4 sm:flex-row">
          <ActionLinks
            href="https://github.com/deven96/ahnlich/releases"
            icon={<DownloadIcon />}>
            Download now
          </ActionLinks>
          <ActionLinks
            href="https://github.com/deven96/ahnlich"
            icon={<GithubIcon />}>
            Star on GitHub
          </ActionLinks>
        </div>
      </div>
    </section>
  );
}

export default function Home(): ReactNode {
  const {siteConfig} = useDocusaurusContext();
  return (
    <Layout
      title={`${siteConfig.title}: In-memory vector database`}
      description={`${siteConfig.tagline}`}>
      <HomepageHeader />
      <main>
        <PythonApi />
        <Highlights />
        <Quickstart />
        <CallToAction />
        <Stats />
      </main>
    </Layout>
  );
}
