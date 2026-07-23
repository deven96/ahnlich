import {useState, type ReactNode} from 'react';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import Link from '@docusaurus/Link';

import Highlights from '@site/src/components/Highlights';
import PythonApi from '@site/src/components/PythonApi';
import HomepageSearchDemo from '@site/src/components/HomepageSearchDemo';

import {ActionLinks} from '../components/buttons';
import {DownloadIcon, GithubIcon, RocketIcon} from '../components/icons';

function HomepageHeader() {
  return (
    <header className="relative overflow-hidden bg-white text-[#45585f] dark:bg-[#08161d] dark:text-white">
      {/* animated background layers */}
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-60 [mask-image:radial-gradient(ellipse_at_center,black,transparent_75%)] dark:opacity-30" />
      <div className="pointer-events-none absolute left-1/2 top-[-10%] h-[38rem] w-[38rem] -translate-x-1/2 rounded-full bg-primary/10 blur-[120px] dark:bg-primary/20" />
      <div className="ahn-blob pointer-events-none absolute -left-32 top-10 h-96 w-96 rounded-full bg-secondary/10 blur-3xl dark:bg-secondary/25" />
      <div
        className="ahn-blob pointer-events-none absolute -right-24 top-32 h-96 w-96 rounded-full bg-primary/10 blur-3xl dark:bg-primary/25"
        style={{animationDelay: '-6s'}}
      />
      <div className="pointer-events-none absolute inset-x-0 bottom-0 h-40 bg-gradient-to-t from-white to-transparent dark:from-[#08161d]" />

      <div className="container relative z-10 flex min-h-screen flex-col items-center justify-center py-20 text-center md:py-32">
        <h1
          className="ahn-hero-title ahn-fade-up max-w-5xl text-3xl font-extrabold leading-[1.08] tracking-tight text-[#0c1e28] dark:text-white md:text-5xl lg:text-6xl"
          style={{animationDelay: '0.1s'}}>
          Add{' '}
          <span className="bg-gradient-to-r from-primary via-secondary to-primary bg-clip-text text-transparent">
            semantic search
          </span>{' '}
          to anything, in minutes
        </h1>



        <div
          className="ahn-fade-up mt-10 flex flex-col items-center gap-4 sm:flex-row"
          style={{animationDelay: '0.22s'}}>
          <ActionLinks href="/docs/getting-started/quickstart" icon={<RocketIcon />}>
            Get Started
          </ActionLinks>
          <ActionLinks href="/docs/overview" variant="ghost">
            View Docs
            <svg
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              strokeWidth={2.4}
              strokeLinecap="round"
              strokeLinejoin="round"
              className="h-4 w-4">
              <path d="m9 18 6-6-6-6" />
            </svg>
          </ActionLinks>
        </div>

        {/* Live Demo integrated into hero */}
        <div
          className="ahn-fade-up mt-16 w-full max-w-3xl"
          style={{animationDelay: '0.28s'}}>
          <div className="mb-4 flex items-center gap-2">
            <span className="inline-block h-2 w-2 animate-pulse rounded-full bg-green-500"></span>
            <span className="text-base font-bold uppercase tracking-wide text-secondary">
              Try it live
            </span>
          </div>
          <HomepageSearchDemo />
          <div className="mt-3 text-xs opacity-50">
            Transformers.js •{' '}
            <Link to="/docs/client-libraries/wasm" className="text-secondary hover:underline">
              Ahnlich WASM
            </Link>
          </div>
        </div>
      </div>

      {/* scroll cue */}
      <div className="pointer-events-none absolute bottom-6 left-1/2 z-10 -translate-x-1/2 text-[#8299a3]/60 dark:text-white/30">
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

type TermLine =
  | {type: 'comment'; text: string}
  | {type: 'cmd'; text: string}
  | {type: 'blank'};

const QUICKSTART_LINES: TermLine[] = [
  {type: 'comment', text: '# Run the Ahnlich DB server'},
  {type: 'cmd', text: 'cargo install ahnlich_db'},
  {type: 'cmd', text: 'ahnlich_db run --port 1369'},
  {type: 'blank'},
  {type: 'comment', text: '# ...or spin up the AI proxy for automatic embeddings'},
  {type: 'cmd', text: 'cargo install ahnlich_ai'},
  {type: 'cmd', text: 'ahnlich_ai run --port 1370'},
];

/** colour a shell command: first word teal, --flags amber, rest default */
function renderCmd(text: string): ReactNode[] {
  return text.split(/(\s+)/).map((tok, i) => {
    if (/^\s+$/.test(tok)) return <span key={i}>{tok}</span>;
    const isFirst = text.trimStart().startsWith(tok) && i === 0;
    const cls = isFirst
      ? 'text-[#6fe0cd] font-semibold'
      : tok.startsWith('--')
        ? 'text-[#e6c07b]'
        : 'text-slate-100';
    return (
      <span key={i} className={cls}>
        {tok}
      </span>
    );
  });
}

function QuickstartTerminal() {
  const [copied, setCopied] = useState(false);
  const onCopy = () => {
    const cmds = QUICKSTART_LINES.filter(
      (l): l is Extract<TermLine, {type: 'cmd'}> => l.type === 'cmd',
    )
      .map((l) => l.text)
      .join('\n');
    navigator.clipboard?.writeText(cmds).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1600);
    });
  };

  return (
    <div className="group overflow-hidden rounded-xl bg-[#0b1f28] shadow-[0_24px_60px_-24px_rgba(8,22,29,0.55)] ring-1 ring-black/5 dark:ring-white/10">
      {/* window header */}
      <div className="flex items-center gap-2 border-b border-solid border-white/[0.07] bg-[#081820] px-4 py-3">
        <span className="h-3 w-3 rounded-full bg-[#ff5f56]" />
        <span className="h-3 w-3 rounded-full bg-[#ffbd2e]" />
        <span className="h-3 w-3 rounded-full bg-[#27c93f]" />
        <span className="ml-2 font-mono text-xs text-white/40">bash — ahnlich</span>
        <button
          onClick={onCopy}
          aria-label={copied ? 'Copied' : 'Copy commands'}
          title={copied ? 'Copied' : 'Copy'}
          className="ml-auto flex h-7 w-7 items-center justify-center rounded-md bg-transparent text-white/45 transition-colors hover:bg-white/10 hover:text-white">
          {copied ? (
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2.5} strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4 text-secondary">
              <path d="M20 6 9 17l-5-5" />
            </svg>
          ) : (
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth={2} strokeLinecap="round" strokeLinejoin="round" className="h-4 w-4">
              <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
              <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
            </svg>
          )}
        </button>
      </div>
      {/* body */}
      <div className="overflow-x-auto p-5 font-mono text-[0.86rem] leading-[1.9] [scrollbar-width:none] [&::-webkit-scrollbar]:hidden">
        {QUICKSTART_LINES.map((line, i) => {
          if (line.type === 'blank') return <div key={i} className="h-3" />;
          if (line.type === 'comment') {
            return (
              <div key={i} className="italic text-[#6d8b99]">
                {line.text}
              </div>
            );
          }
          const isLast = i === QUICKSTART_LINES.length - 1;
          return (
            <div key={i} className="whitespace-pre">
              <span className="mr-2 select-none text-secondary">$</span>
              {renderCmd(line.text)}
              {isLast && (
                <span className="ml-1 inline-block h-4 w-2 translate-y-[3px] animate-pulse bg-secondary/80" />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function Quickstart() {
  return (
    <section className="bg-[#f4f8fa] py-24 dark:bg-[#0c1e28]">
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
            <ActionLinks href="/docs/getting-started/quickstart" icon={<RocketIcon />}>
              Read the Quickstart
            </ActionLinks>
            <Link
              to="/docs/client-libraries"
              className="text-lg font-medium text-primary hover:text-primary/75">
              Explore client SDKs →
            </Link>
          </div>
        </div>
        <QuickstartTerminal />
      </div>
    </section>
  );
}

function CallToAction() {
  return (
    <section className="relative overflow-hidden bg-[#f4f8fa] py-24 text-center text-[#45585f] dark:bg-[#0a1a22] dark:text-white">
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-50 dark:opacity-30" />
      <div className="pointer-events-none absolute left-1/2 top-0 h-72 w-[36rem] -translate-x-1/2 rounded-full bg-secondary/10 blur-3xl dark:bg-secondary/20" />
      <div className="container relative z-10">
        <h2 className="text-3xl font-bold text-[#0c1e28] dark:text-white md:text-4xl">
          Start building smarter search today
        </h2>
        <p className="mx-auto mt-4 max-w-xl text-lg text-[#45585f] dark:text-white/80">
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
            variant="ghost"
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
      </main>
    </Layout>
  );
}
