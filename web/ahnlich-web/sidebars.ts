import type { SidebarsConfig } from '@docusaurus/plugin-content-docs';

// This runs in Node.js - Don't use client-side code here (browser APIs, JSX...)

/**
 * Creating a sidebar enables you to:
 - create an ordered group of docs
 - render a sidebar for each doc of that group
 - provide next/previous navigation

 The sidebars can be generated from the filesystem, or explicitly defined here.

 Create as many sidebars as you want.
 */
const sidebars: SidebarsConfig = {
  docsSidebar: [
    "overview",
    {
      type: 'category',
      label: '🚀 Getting Started',
      link: {
        type: 'doc',
        id: 'getting-started/getting-started',
      },
      items: [
        'getting-started/installation',
        'getting-started/usage',
        'getting-started/comparison-with-other-tools',
        'getting-started/next-steps',
      ],
    },
    {
      type: 'category',
      label: '🧩 Components',
      link: {
        type: 'doc',
        id: 'components/components',
      },
      items: [
        {
          type: 'category',
          label: '📟 Ahnlich CLI',
          link: {
            type: 'doc',
            id: 'components/ahnlich-cli/ahnlich-cli'
          },
          items: [
            'components/ahnlich-cli/installation',
            'components/ahnlich-cli/db-commands',
            'components/ahnlich-cli/ai-commands',
          ],
        },
        {
          type: 'category',
          label: '🗄️ Ahnlich DB',
          link: {
            type: 'doc',
            id: 'components/ahnlich-db/ahnlich-db'
          },
          items: [
            'components/ahnlich-db/use-cases',
            'components/ahnlich-db/installation',
            'components/ahnlich-db/reference',
            'components/ahnlich-db/advanced',
          ],
        },
        {
          type: 'category',
          label: '🤖 Ahnlich AI',
          link: {
            type: 'doc',
            id: 'components/ahnlich-ai/ahnlich-ai'
          },
          items: [
            'components/ahnlich-ai/use-cases',
            'components/ahnlich-ai/setup-config',
            'components/ahnlich-ai/reference',
            'components/ahnlich-ai/advanced',
            'components/ahnlich-ai/deep-dive'
          ],
        },
        {
          type: 'category',
          label: '♾️ Persistence In Ahnlich',
          link: {
            type: 'doc',
            id: 'components/persistence-in-ahnlich/persistence-in-ahnlich'
          },
          items: [
            'components/persistence-in-ahnlich/persistence-for-ahnlich-db',
            'components/persistence-in-ahnlich/persistence-for-ahnlich-ai',
          ],
        },
        {
          type: 'category',
          label: '🕸️ Distributed Tracing',
          link: {
            type: 'doc',
            id: 'components/distributed-tracing/distributed-tracing'
          },
          items: [
            'components/distributed-tracing/ahnlich-db',
            'components/distributed-tracing/ahnlich-ai',
            'components/distributed-tracing/using-jaeger',
          ],
        },
      ],
    },
    {
      type: 'category',
      label: '📚 Client Libraries',
      link: {
        type: 'doc',
        id: 'client-libraries/client-libraries'
      },
      items: [
        {
          type: 'category',
          label: '🐍 Python',
          link: {
            type: 'doc',
            id: 'client-libraries/python/python'
          },
          items: [
            'client-libraries/python/python-specific-resources',
            {
              type: 'category',
              label: 'Request DB',
              link: {
                type: 'doc',
                id: 'client-libraries/python/request-db/request-db'
              },
              items: [
                'client-libraries/python/request-db/ping',
                'client-libraries/python/request-db/info-server',
                'client-libraries/python/request-db/list-stores',
                'client-libraries/python/request-db/create-store',
                'client-libraries/python/request-db/set',
                'client-libraries/python/request-db/get-simn',
                'client-libraries/python/request-db/get-key',
                'client-libraries/python/request-db/get-by-predicate',
                'client-libraries/python/request-db/create-predicate-index',
                'client-libraries/python/request-db/drop-predicate-index',
                'client-libraries/python/request-db/delete-key',
                'client-libraries/python/request-db/drop-store',
                'client-libraries/python/request-db/create-non-linear-algx',
                'client-libraries/python/request-db/drop-non-linear-algx',
                'client-libraries/python/request-db/delete-predicate',
              ]
            },
            {
              type: 'category',
              label: 'Request AI',
              link: {
                type: 'doc',
                id: 'client-libraries/python/request-ai/request-ai'
              },
              items: [
                'client-libraries/python/request-ai/ping',
                'client-libraries/python/request-ai/info-server',
                'client-libraries/python/request-ai/list-stores',
                'client-libraries/python/request-ai/create-store',
                'client-libraries/python/request-ai/set',
                'client-libraries/python/request-ai/get-simn',
                'client-libraries/python/request-ai/get-by-predicate',
                'client-libraries/python/request-ai/create-predicate-index',
                'client-libraries/python/request-ai/drop-predicate-index',
                'client-libraries/python/request-ai/delete-key',
                'client-libraries/python/request-ai/drop-store',
                'client-libraries/python/request-ai/create-non-linear-algx',
                'client-libraries/python/request-ai/drop-non-linear-algx',
              ]
            },
            'client-libraries/python/bulk-requests',
            'client-libraries/python/type-meanings'
          ],
        },
        {
          type: 'category',
          label: '⚙️ Go',
          link: {
            type: 'doc',
            id: 'client-libraries/go/go',
          },
          items: [
            'client-libraries/go/go-specific-resources',
            {
              type: 'category',
              label: 'Request DB',
              link: {
                type: 'doc',
                id: 'client-libraries/go/request-db/request-db'
              },
              items: [
                'client-libraries/go/request-db/ping',
                'client-libraries/go/request-db/info-server',
                'client-libraries/go/request-db/list-stores',
                'client-libraries/go/request-db/create-store',
                'client-libraries/go/request-db/set',
                'client-libraries/go/request-db/get-simn',
                'client-libraries/go/request-db/get-key',
                'client-libraries/go/request-db/get-by-predicate',
                'client-libraries/go/request-db/create-predicate-index',
                'client-libraries/go/request-db/drop-predicate-index',
                'client-libraries/go/request-db/delete-key',
                'client-libraries/go/request-db/drop-store',
                'client-libraries/go/request-db/list-connected-clients',
                'client-libraries/go/request-db/create-non-linear-algx',
                'client-libraries/go/request-db/drop-non-linear-algx',
                'client-libraries/go/request-db/delete-predicate',
              ]
            },
            {
              type: 'category',
              label: 'Request AI',
              link: {
                type: 'doc',
                id: 'client-libraries/go/request-ai/request-ai'
              },
              items: [
                'client-libraries/go/request-ai/ping',
                'client-libraries/go/request-ai/info-server',
                'client-libraries/go/request-ai/list-stores',
                'client-libraries/go/request-ai/create-store',
                'client-libraries/go/request-ai/set',
                'client-libraries/go/request-ai/get-simn',
                'client-libraries/go/request-ai/get-by-predicate',
                'client-libraries/go/request-ai/create-predicate-index',
                'client-libraries/go/request-ai/drop-predicate-index',
                'client-libraries/go/request-ai/delete-key',
                'client-libraries/go/request-ai/drop-store',
                'client-libraries/go/request-ai/create-non-linear-algx',
                'client-libraries/go/request-ai/drop-non-linear-algx',
              ]
            },
            'client-libraries/go/bulk-requests',
            'client-libraries/go/type-meanings'
          ]
        },
        {
          type: 'category',
          label: '⚙️ Rust',
          link: {
            type: 'doc',
            id: 'client-libraries/rust/rust',
          },
          items: [
            'client-libraries/rust/rust-specific-resources',
            {
              type: 'category',
              label: 'Request DB',
              link: {
                type: 'doc',
                id: 'client-libraries/rust/request-db/request-db'
              },
              items: [
                'client-libraries/rust/request-db/ping',
                'client-libraries/rust/request-db/info-server',
                'client-libraries/rust/request-db/list-stores',
                'client-libraries/rust/request-db/create-store',
                'client-libraries/rust/request-db/set',
                'client-libraries/rust/request-db/get-simn',
                'client-libraries/rust/request-db/get-key',
                'client-libraries/rust/request-db/get-by-predicate',
                'client-libraries/rust/request-db/create-predicate-index',
                'client-libraries/rust/request-db/drop-predicate-index',
                'client-libraries/rust/request-db/delete-key',
                'client-libraries/rust/request-db/drop-store',
                'client-libraries/rust/request-db/list-connected-clients',
                'client-libraries/rust/request-db/create-non-linear-algx',
                'client-libraries/rust/request-db/drop-non-linear-algx',
                'client-libraries/rust/request-db/delete-by-predicate'
              ]
            },
            {
              type: 'category',
              label: 'Request AI',
              link: {
                type: 'doc',
                id: 'client-libraries/rust/request-ai/request-ai'
              },
              items: [
                'client-libraries/rust/request-ai/ping',
                'client-libraries/rust/request-ai/info-server',
                'client-libraries/rust/request-ai/list-stores',
                'client-libraries/rust/request-ai/create-store',
                'client-libraries/rust/request-ai/set',
                'client-libraries/rust/request-ai/get-simn',
                'client-libraries/rust/request-ai/get-key',
                'client-libraries/rust/request-ai/get-by-predicate',
                'client-libraries/rust/request-ai/create-predicate-index',
                'client-libraries/rust/request-ai/drop-predicate-index',
                'client-libraries/rust/request-ai/delete-key',
                'client-libraries/rust/request-ai/drop-store',
                'client-libraries/rust/request-ai/list-connected-clients',
                'client-libraries/rust/request-ai/create-non-linear-algx',
                'client-libraries/rust/request-ai/drop-non-linear-algx',
                'client-libraries/rust/request-ai/new',
                'client-libraries/rust/request-ai/purge-stores'
              ]
            },
            'client-libraries/rust/pipeline',
            'client-libraries/rust/types-and-utilities',
            'client-libraries/rust/testing',
            'client-libraries/rust/distributed-tracing'
          ]
        },
      ],
    },
    {
      type: 'category',
      label: '⚡Ahnlich in production',
      items: [
        {
          type: 'autogenerated',
          dirName: 'ahnlich-in-production',
        },
      ],
    },
    "architecture",
    "community",
  ],
};

export default sidebars;
