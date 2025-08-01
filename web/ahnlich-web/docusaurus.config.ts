import {themes as prismThemes} from 'prism-react-renderer';
import type {Config} from '@docusaurus/types';
import * as Preset from '@docusaurus/preset-classic';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

const config: Config = {
  title: 'Ahnlich',
  tagline: 'A project by developers bringing vector database and artificial intelligence powered semantic search abilities closer to you',
  favicon: 'img/logo.jpg',

  // Set the production url of your site here
  url: 'https://ahnlich.tech',
  // Set the /<baseUrl>/ pathname under which your site is served
  // For GitHub pages deployment, it is often '/<projectName>/'
  baseUrl: '/',

  // GitHub pages deployment config.
  // If you aren't using GitHub pages, you don't need these.
  organizationName: 'deven96', // Usually your GitHub org/user name.
  projectName: 'ahnlich', // Usually your repo name.

  onBrokenLinks: 'throw',
  onBrokenMarkdownLinks: 'throw',

  // Even if you don't use internationalization, you can use this field to set
  // useful metadata like html lang. For example, if your site is Chinese, you
  // may want to replace "en" with "zh-Hans".
  i18n: {
    defaultLocale: 'en',
    locales: ['en'],
  },

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
      } satisfies Preset.Options,
    ],
  ],

  themeConfig: {
    // Replace with your project's social card
    metadata: [
      { name: 'twitter:image', content: 'img/logo.jpg' },
      { property: 'og:image', content: 'img/logo.jpg' },
    ],
    image: 'img/logo.jpg',

    navbar: {
      title: 'AHNLICH',
      logo: {
        alt: 'Ahnlich Logo',
        src: 'img/logo.jpg',
      },
      items: [
        {
          type: 'docSidebar',
          sidebarId: 'docsSidebar',
          position: 'right',
          label: 'Docs',
        },
        {to: '/blog', label: 'Blog', position: 'right'},
        {to: '/docs/guides', label: 'Guides', position: 'right'},
        {
          href: 'https://github.com/deven96/ahnlich',
          label: 'GitHub',
          position: 'right',
        },
      ],
    },
    footer: {
      style: 'dark',
      links: [
        {
          title: 'Docs',
          items: [
            {
              label: 'Guides',
              to: '/docs/guides',
            },
            {
              label: 'Overview',
              to: '/docs/overview',
            },
            {
              label: 'Getting Started',
              to: '/docs/getting-started',
            },
            {
              label: 'Components',
              to: '/docs/components',
            },
            {
              label: 'Client Libraries',
              to: '/docs/client-libraries',
            },
            {
              label: 'Ahnlich In Production',
              to: '/docs/ahnlich-in-production',
            },
            {
              label: 'Architecture',
              to: '/docs/architecture',
            }
          ],
        },
        {
          title: 'Community',
          items: [
            {
              label: 'WhatsApp',
              href: 'https://whatsapp.com',
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
