import type {ReactNode} from 'react';
import {useLocation} from '@docusaurus/router';
import Link from '@docusaurus/Link';

type Tab = {label: string; to: string; match: string[]};

const TABS: Tab[] = [
  {
    label: 'Getting Started',
    to: '/docs/overview',
    match: ['/docs/overview', '/docs/getting-started'],
  },
  {
    label: 'Vector DB',
    to: '/docs/components/ahnlich-db',
    match: ['/docs/components/ahnlich-db'],
  },
  {
    label: 'AI',
    to: '/docs/components/ahnlich-ai',
    match: ['/docs/components/ahnlich-ai'],
  },
  {
    label: 'CLI',
    to: '/docs/components/ahnlich-cli',
    match: ['/docs/components/ahnlich-cli'],
  },
  {
    label: 'Clients',
    to: '/docs/client-libraries',
    match: ['/docs/client-libraries'],
  },
  {label: 'Guides', to: '/docs/guides', match: ['/docs/guides']},
  {
    label: 'Reference',
    to: '/docs/components',
    match: [
      '/docs/components',
      '/docs/components/schemas',
      '/docs/components/predicates',
      '/docs/components/persistence-in-ahnlich',
      '/docs/components/distributed-tracing',
      '/docs/ahnlich-in-production',
      '/docs/reference',
      '/docs/troubleshooting',
      '/docs/architecture',
      '/docs/community',
    ],
  },
];

/**
 * Prisma-style secondary docs navigation: a horizontal tab bar that sits
 * directly under the main navbar on docs pages.
 */
export default function DocsSubnav(): ReactNode {
  const {pathname} = useLocation();

  // Longest matching prefix wins, so e.g. /docs/components/ahnlich-db activates
  // "Vector DB" rather than "Reference" (which owns the broader /docs/components).
  let activeIdx = -1;
  let bestLen = -1;
  TABS.forEach((tab, i) => {
    tab.match.forEach((m) => {
      if (
        (pathname === m || pathname.startsWith(m + '/')) &&
        m.length > bestLen
      ) {
        bestLen = m.length;
        activeIdx = i;
      }
    });
  });

  return (
    <nav className="ahn-docsnav" aria-label="Docs sections">
      <div className="ahn-docsnav__inner container">
        {TABS.map((tab, i) => {
          const active = i === activeIdx;
          return (
            <Link
              key={tab.label}
              to={tab.to}
              className={`ahn-docsnav__tab${
                active ? ' ahn-docsnav__tab--active' : ''
              }`}>
              {tab.label}
            </Link>
          );
        })}
      </div>
    </nav>
  );
}
