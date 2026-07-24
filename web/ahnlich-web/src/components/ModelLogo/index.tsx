import type {ReactNode} from 'react';

/*
 * Small inline source logos for the model tables, sized to sit on a line of text.
 * Used only as source-attribution marks next to a link to the model's page.
 */

const base = {
  display: 'inline-block',
  width: '1.1em',
  height: '1.1em',
  verticalAlign: '-0.2em',
  marginRight: '0.35em',
  flexShrink: 0,
} as const;

/** GitHub mark (monochrome, follows text colour). */
export function GitHubLogo(): ReactNode {
  return (
    <svg style={base} viewBox="0 0 24 24" fill="currentColor" role="img" aria-label="GitHub">
      <path d="M12 .297c-6.63 0-12 5.373-12 12 0 5.303 3.438 9.8 8.205 11.385.6.113.82-.258.82-.577 0-.285-.01-1.04-.015-2.04-3.338.724-4.042-1.61-4.042-1.61C4.422 18.07 3.633 17.7 3.633 17.7c-1.087-.744.084-.729.084-.729 1.205.084 1.838 1.236 1.838 1.236 1.07 1.835 2.809 1.305 3.495.998.108-.776.417-1.305.76-1.605-2.665-.3-5.466-1.332-5.466-5.93 0-1.31.465-2.38 1.235-3.22-.135-.303-.54-1.523.105-3.176 0 0 1.005-.322 3.3 1.23.96-.267 1.98-.399 3-.405 1.02.006 2.04.138 3 .405 2.28-1.552 3.285-1.23 3.285-1.23.645 1.653.24 2.873.12 3.176.765.84 1.23 1.91 1.23 3.22 0 4.61-2.805 5.625-5.475 5.92.42.36.81 1.096.81 2.22 0 1.606-.015 2.896-.015 3.286 0 .315.21.69.825.57C20.565 22.092 24 17.592 24 12.297c0-6.627-5.373-12-12-12" />
    </svg>
  );
}

/** Hugging Face mark (their yellow smiling face). */
export function HFLogo(): ReactNode {
  return (
    <svg style={base} viewBox="0 0 32 32" role="img" aria-label="Hugging Face">
      <circle cx="16" cy="17" r="12" fill="#FFD21E" />
      <circle cx="11.4" cy="15" r="1.7" fill="#3A2F0B" />
      <circle cx="20.6" cy="15" r="1.7" fill="#3A2F0B" />
      <path
        d="M10.4 19.5 Q16 25.5 21.6 19.5"
        fill="none"
        stroke="#3A2F0B"
        strokeWidth="1.9"
        strokeLinecap="round"
      />
      <circle cx="8.4" cy="18.6" r="1.4" fill="#FF9D0B" opacity="0.7" />
      <circle cx="23.6" cy="18.6" r="1.4" fill="#FF9D0B" opacity="0.7" />
    </svg>
  );
}
