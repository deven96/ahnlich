import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import * as Preset from '@docusaurus/preset-classic';

require('dotenv').config();

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Ahnlich',
  tagline: 'A project by developers bringing vector database and artificial intelligence powered semantic search abilities closer to you',
  favicon: 'img/logo.png',
  markdown: {
      mermaid: true,
    },
    themes: [
      '@docusaurus/theme-mermaid',
      [
        require.resolve("@easyops-cn/docusaurus-search-local"),
        {
          hashed: true,
          language: ["en"],
          highlightSearchTermsOnTargetPage: true,
          explicitSearchResultPath: true,
          docsRouteBasePath: '/docs',
        },
      ],
    ],

  // Set the production url of your site here
  url: 'https://ahnlich.dev',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'deven96', // Usually your GitHub org/user name.
  projectName: 'ahnlich', // Usually your repo name.

  // FIXME: Turn these back to 'throw' once docs site is complete
  onBrokenLinks: 'warn',
  onBrokenMarkdownLinks: 'warn',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

  stylesheets: [
    {
      href: 'https://fonts.googleapis.com/css2?family=Unbounded:wght@600;700;800&display=swap',
      type: 'text/css',
    },
  ],

  plugins: [
    async function myPlugin(context, options) {
      return {
        name: "docusaurus-tailwindcss",
        configurePostCss(postcssOptions) {
          // Appends TailwindCSS and AutoPrefixer.
          postcssOptions.plugins.push(require("tailwindcss"));
          postcssOptions.plugins.push(require("autoprefixer"));
          return postcssOptions;
        },
      };
    },
    // Generate machine-readable docs for LLMs: /llms.txt (index) and
    // /llms-full.txt (all docs concatenated as plain markdown).
    function llmsTxtPlugin(context) {
      return {
        name: "ahnlich-llms-txt",
        async postBuild({ siteConfig, routesPaths, outDir }) {
          const fs = require("fs");
          const path = require("path");
          const base = siteConfig.url.replace(/\/$/, "");

          const SECTIONS: [string, string][] = [
            ["getting-started", "Getting started"],
            ["concepts", "Concepts"],
            ["stores", "Stores"],
            ["components", "Components"],
            ["client-libraries", "Client libraries"],
            ["guides", "Guides"],
            ["reference", "Reference"],
            ["troubleshooting", "Troubleshooting"],
            ["ahnlich-in-production", "Operations"],
          ];

          const docRoutes = routesPaths
            .filter((r: string) => r.startsWith("/docs/") && r !== "/docs/")
            .sort();

          const titleOf = (route: string) => {
            const html = path.join(outDir, route.replace(/^\//, ""), "index.html");
            try {
              const s = fs.readFileSync(html, "utf8");
              const decode = (x: string) =>
                x
                  .replace(/&amp;/g, "&")
                  .replace(/&lt;/g, "<")
                  .replace(/&gt;/g, ">")
                  .replace(/&quot;/g, '"')
                  .replace(/&#x27;|&#39;/g, "'")
                  .replace(/^\p{Extended_Pictographic}️?\s*/u, "");
              const t = (s.match(/<title[^>]*>([^<]*)<\/title>/) || [])[1] || route;
              const d = (s.match(/<meta name="description" content="([^"]*)"/) || [])[1] || "";
              return { title: decode(t.replace(/\s*\|.*$/, "").trim()), desc: decode(d) };
            } catch {
              return { title: route, desc: "" };
            }
          };

          const lines: string[] = [
            `# ${siteConfig.title}`,
            "",
            `> ${siteConfig.tagline}`,
            "",
            "This is the documentation index for LLMs. Fetch the linked pages as needed.",
            "",
          ];
          const used = new Set<string>();
          for (const [seg, label] of SECTIONS) {
            const rs = docRoutes.filter(
              (r: string) => r === `/docs/${seg}` || r.startsWith(`/docs/${seg}/`),
            );
            if (!rs.length) continue;
            lines.push(`## ${label}`, "");
            for (const r of rs) {
              used.add(r);
              const { title, desc } = titleOf(r);
              lines.push(`- [${title}](${base}${r})${desc ? `: ${desc}` : ""}`);
            }
            lines.push("");
          }
          const rest = docRoutes.filter((r: string) => !used.has(r));
          if (rest.length) {
            lines.push("## Other", "");
            for (const r of rest) {
              const { title } = titleOf(r);
              lines.push(`- [${title}](${base}${r})`);
            }
            lines.push("");
          }
          fs.writeFileSync(path.join(outDir, "llms.txt"), lines.join("\n"));

          // llms-full.txt — every docs source file concatenated.
          const docsDir = path.join(context.siteDir, "docs");
          const full: string[] = [`# ${siteConfig.title} — full documentation`, ""];
          const walk = (dir: string) => {
            for (const e of fs.readdirSync(dir, { withFileTypes: true }).sort((a, b) =>
              a.name.localeCompare(b.name),
            )) {
              const fp = path.join(dir, e.name);
              if (e.isDirectory()) walk(fp);
              else if (/\.mdx?$/.test(e.name)) {
                const rel = path.relative(docsDir, fp);
                full.push(`\n\n---\n\n# FILE: ${rel}\n`, fs.readFileSync(fp, "utf8"));
              }
            }
          };
          walk(docsDir);
          fs.writeFileSync(path.join(outDir, "llms-full.txt"), full.join("\n"));
        },
      };
    },
  ],

  presets: [
    [
      'classic',
      {
        docs: {
          sidebarPath: './sidebars.ts',
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: ({locale, docPath}) => {
            return `https://github.com/deven96/ahnlich/tree/main/web/ahnlich-web/docs/${docPath}`;
          },
        },
        blog: {
          showReadingTime: true,
          feedOptions: {
            type: ['rss', 'atom'],
            xslt: true,
          },
          // Please change this to your repo.
          // Remove this to remove the "edit this page" links.
          editUrl: ({locale, blogPath}) => {
            return `https://github.com/deven96/ahnlich/tree/main/web/ahnlich-web/blog/${blogPath}`;
          },
          // Useful options to enforce blogging best practices
          onInlineTags: 'warn',
          onInlineAuthors: 'warn',
          onUntruncatedBlogPosts: 'warn',
        },
        theme: {
          customCss: './src/css/custom.css',
        },
        ...(process.env.G_TRACKING_ID
          ? {
              gtag: {
                trackingID: process.env.G_TRACKING_ID,
                anonymizeIP: true,
              },
            }
          : {}),
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    colorMode: {
      defaultMode: 'light',
      respectPrefersColorScheme: true,
    },
    docs: {
      sidebar: {
        // adds the collapse/expand (hide) toggle at the bottom of the sidebar
        hideable: true,
        // keep the tree tidy: opening one category collapses the others
        autoCollapseCategories: true,
      },
    },
    mermaid: {
      theme: {light: 'neutral', dark: 'dark'},
    },
    metadata: [
      { name: 'twitter:image', content: 'img/logo.jpg' },
      { property: 'og:image', content: 'img/logo.jpg' },
    ],
    image: 'img/logo.png',

    navbar: {
      title: 'AHNLICH',
      logo: {
        alt: 'Ahnlich Logo',
        src: 'img/logo.png',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'gettingStartedSidebar',
          position: 'right',
          label: 'Docs',
        },
        {to: '/blog', label: 'Blog', position: 'right'},
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Community',
          items: [
            {
              label: 'WhatsApp',
              href: 'https://chat.whatsapp.com/E4CP7VZ1lNH9dJUxpsZVvD',
            },
            {
              label: 'GitHub Discussions',
              href: 'https://github.com/deven96/ahnlich/discussions',
            }
          ],
        },
        {
          title: 'More',
          items: [
            {
              label: 'Blog',
              to: '/blog',
            },
            {
              label: 'GitHub',
              href: 'https://github.com/deven96/ahnlich',
            },
          ],
        },
      ],
      copyright: `Copyright © ${new Date().getFullYear()} |  Ahnlich | Built with Docusaurus.`,
    },
    prism: {
      theme: prismThemes.github,
      darkTheme: prismThemes.dracula,
    },
  } satisfies Preset.ThemeConfig,
};

export default config;