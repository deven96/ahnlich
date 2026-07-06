import type {ReactNode} from 'react';
import Link from '@docusaurus/Link';
import useBaseUrl from '@docusaurus/useBaseUrl';


const COLUMNS = [
  {
    title: 'Product',
    links: [
      {label: 'Get Started', to: '/docs/getting-started'},
      {label: 'Guides', to: '/docs/guides'},
      {label: 'Client Libraries', to: '/docs/client-libraries'},
      {label: 'Blog', to: '/blog'},
    ],
  },
  {
    title: 'Community',
    links: [
      {
        label: 'WhatsApp',
        href: 'https://chat.whatsapp.com/E4CP7VZ1lNH9dJUxpsZVvD',
      },
      {
        label: 'GitHub Discussions',
        href: 'https://github.com/deven96/ahnlich/discussions',
      },
      {label: 'GitHub', href: 'https://github.com/deven96/ahnlich'},
      {
        label: 'Releases',
        href: 'https://github.com/deven96/ahnlich/releases',
      },
    ],
  },
];

/**
 * Marketing-style footer used only on the landing page, matching the dark
 * hero aesthetic. Docs/blog keep the default Docusaurus footer.
 */
export default function HomeFooter(): ReactNode {
  const logo = useBaseUrl('/img/logo.png');
  const year = new Date().getFullYear();

  return (
    <footer className="relative overflow-hidden border-t border-solid border-[#e0eaef] bg-[#f4f8fa] text-[#45585f] dark:border-transparent dark:bg-[#08161d] dark:text-white">
      <div className="ahn-grid pointer-events-none absolute inset-0 opacity-40 [mask-image:radial-gradient(ellipse_at_top,black,transparent_75%)] dark:opacity-20" />
      <div className="pointer-events-none absolute left-1/2 top-0 h-40 w-[40rem] -translate-x-1/2 rounded-full bg-primary/5 blur-3xl dark:bg-primary/10" />

      <div className="container relative z-10 py-16">
        <div className="grid gap-12 md:grid-cols-[1.5fr_1fr_1fr]">
          {/* brand */}
          <div className="max-w-sm">
            <Link
              to="/"
              className="flex items-center gap-2 text-[#0c1e28] no-underline hover:text-[#0c1e28] dark:text-white dark:hover:text-white">
              <img src={logo} alt="Ahnlich logo" className="h-9 w-9" />
              <span className="text-lg font-bold tracking-tight">AHNLICH</span>
            </Link>
            <p className="mt-4 text-sm leading-relaxed text-[#45585f] dark:text-white/60">
              A high-performance, AI-powered in-memory vector database for
              semantic search. Lightning fast, developer friendly, and open
              source.
            </p>
          </div>

          {/* link columns */}
          {COLUMNS.map((col) => (
            <div key={col.title}>
              <h3 className="mb-4 text-sm font-semibold uppercase tracking-wider text-[#8299a3] dark:text-white/50">
                {col.title}
              </h3>
              <ul className="flex list-none flex-col gap-3 p-0">
                {col.links.map((link) => (
                  <li key={link.label}>
                    <Link
                      to={'to' in link ? link.to : undefined}
                      href={'href' in link ? link.href : undefined}
                      className="text-sm text-[#45585f] no-underline transition-colors hover:text-primary dark:text-white/70 dark:hover:text-secondary">
                      {link.label}
                    </Link>
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>

        <div className="mt-14 border-t border-solid border-[#e0eaef] pt-8 text-sm text-[#8299a3] dark:border-white/10 dark:text-white/40">
          Copyright © {year} | Ahnlich | Built with Docusaurus.
        </div>
      </div>
    </footer>
  );
}
