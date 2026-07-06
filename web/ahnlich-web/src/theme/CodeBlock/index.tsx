import React, {useState, type ReactNode} from 'react';
import OriginalCodeBlock from '@theme-original/CodeBlock';
import type CodeBlockType from '@theme/CodeBlock';
import type {WrapperProps} from '@docusaurus/types';

type Props = WrapperProps<typeof CodeBlockType>;

// Prettier display names for common languages; falls back to UPPERCASE.
const LANGUAGE_LABELS: Record<string, string> = {
  rs: 'Rust',
  rust: 'Rust',
  py: 'Python',
  python: 'Python',
  ts: 'TypeScript',
  typescript: 'TypeScript',
  js: 'JavaScript',
  javascript: 'JavaScript',
  go: 'Go',
  golang: 'Go',
  sh: 'Shell',
  bash: 'Shell',
  shell: 'Shell',
  zsh: 'Shell',
  json: 'JSON',
  yaml: 'YAML',
  yml: 'YAML',
  toml: 'TOML',
  proto: 'Proto',
  protobuf: 'Proto',
  sql: 'SQL',
  dockerfile: 'Docker',
  docker: 'Docker',
  text: 'Text',
  plaintext: 'Text',
};

const TERMINAL_LANGS = new Set([
  'bash',
  'sh',
  'shell',
  'zsh',
  'console',
  'shell-session',
  'shellsession',
  'terminal',
]);

function rawLanguage(props: Props): string {
  const fromClass = /language-([\w-]+)/.exec(props.className ?? '')?.[1];
  return (fromClass ?? (props as {language?: string}).language ?? '')
    .toLowerCase()
    .trim();
}

function resolveLanguage(props: Props): string | null {
  const raw = rawLanguage(props);
  if (!raw || raw === 'none' || raw === 'mermaid') {
    return null;
  }
  return LANGUAGE_LABELS[raw] ?? raw.toUpperCase();
}

function hasTitle(props: Props): boolean {
  return Boolean(
    (props as {title?: string}).title ||
      /(^|\s)title=/.test((props as {metastring?: string}).metastring ?? ''),
  );
}

function WindowDots(): ReactNode {
  return (
    <span className="ahn-codeblock__dots" aria-hidden="true">
      <span />
      <span />
      <span />
    </span>
  );
}

function TerminalCopyButton({code}: {code: string}): ReactNode {
  const [copied, setCopied] = useState(false);
  const onCopy = () => {
    navigator.clipboard?.writeText(code).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1500);
    });
  };
  return (
    <button
      type="button"
      className="ahn-terminal__copy"
      aria-label={copied ? 'Copied' : 'Copy command'}
      title={copied ? 'Copied' : 'Copy'}
      onClick={onCopy}>
      {copied ? (
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round">
          <path d="M20 6 9 17l-5-5" />
        </svg>
      ) : (
        <svg viewBox="0 0 24 24" width="15" height="15" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
          <rect width="14" height="14" x="8" y="8" rx="2" ry="2" />
          <path d="M4 16c-1.1 0-2-.9-2-2V4c0-1.1.9-2 2-2h10c1.1 0 2 .9 2 2" />
        </svg>
      )}
    </button>
  );
}

/**
 * Custom renderer for shell/command blocks. Each *new* command gets a `$`
 * prompt; continuation lines (the previous line ended with `\`), comments and
 * blank lines do not — so multi-line commands read as one command.
 */
const SHELL_OPERATORS = new Set([
  '|', '||', '&&', ';', '&', '>', '>>', '<', '\\', '$(', ')',
]);

/** Lightweight bash tokeniser → returns a CSS-class suffix for each token. */
function classifyToken(tok: string): string {
  if (/^["']/.test(tok)) return 'str';
  if (/^-{1,2}[A-Za-z0-9]/.test(tok)) return 'flag';
  if (/:\/\//.test(tok) || /^[\w.@-]+\.(io|com|dev|org|net)(\/|:|$)/.test(tok)) {
    return 'url';
  }
  if (SHELL_OPERATORS.has(tok)) return 'op';
  return 'txt';
}

/** Split a line into coloured spans, colouring the first word as the command. */
function renderBashLine(line: string, isCommand: boolean): ReactNode[] {
  const parts = line.match(/(\s+|"[^"]*"|'[^']*'|[^\s]+)/g) ?? [line];
  let firstWord = true;
  return parts.map((part, idx) => {
    if (/^\s+$/.test(part)) {
      return <span key={idx}>{part}</span>;
    }
    const kind =
      isCommand && firstWord && !SHELL_OPERATORS.has(part)
        ? 'cmd'
        : classifyToken(part);
    firstWord = false;
    return (
      <span key={idx} className={`ahn-terminal__${kind}`}>
        {part}
      </span>
    );
  });
}

function Terminal({code}: {code: string}): ReactNode {
  const lines = code.replace(/\n+$/, '').split('\n');
  let prevContinues = false;

  return (
    <div className="ahn-codeblock ahn-codeblock--terminal">
      <TerminalCopyButton code={code} />
      <pre className="ahn-terminal">
        <code>
          {lines.map((line, i) => {
            const trimmed = line.trim();
            const isComment = trimmed.startsWith('#');
            const isBlank = trimmed === '';
            const showPrompt = !isBlank && !isComment && !prevContinues;
            prevContinues = trimmed.endsWith('\\');
            return (
              <span
                key={i}
                className={`ahn-terminal__line${
                  isComment ? ' ahn-terminal__line--comment' : ''
                }`}>
                {showPrompt && (
                  <span className="ahn-terminal__prompt" aria-hidden="true">
                    $
                  </span>
                )}
                <span className="ahn-terminal__text">{isComment || isBlank ? (line || " ") : renderBashLine(line, showPrompt)}</span>
              </span>
            );
          })}
        </code>
      </pre>
    </div>
  );
}

export default function CodeBlockWrapper(props: Props): ReactNode {
  const language = resolveLanguage(props);
  const isTerminal = TERMINAL_LANGS.has(rawLanguage(props));
  const rawCode =
    typeof props.children === 'string' ? props.children : null;

  // Shell/command blocks use our own prompted terminal renderer.
  if (isTerminal && rawCode !== null) {
    return <Terminal code={rawCode} />;
  }

  const titled = hasTitle(props);
  // Source files with no title get a window header (dots + language label).
  const ownHeader = !titled;
  const headerLabel = language;

  const classes = [
    'ahn-codeblock',
    ownHeader && 'ahn-codeblock--own-header',
    titled && 'ahn-codeblock--titled',
    isTerminal && 'ahn-codeblock--terminal',
  ]
    .filter(Boolean)
    .join(' ');

  return (
    <div className={classes}>
      {ownHeader && (
        <div className="ahn-codeblock__header">
          <WindowDots />
          {headerLabel && (
            <span className="ahn-codeblock__lang">{headerLabel}</span>
          )}
        </div>
      )}
      {/* Line numbers on by default for source files, but never for terminals.
          A block can still opt out with showLineNumbers={false}. */}
      <OriginalCodeBlock showLineNumbers={!isTerminal} {...props} />
    </div>
  );
}
