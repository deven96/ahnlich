---
id: getting-started
title: 🚀 Getting started
---

import CustomDocCard from '@site/src/components/CustomDocCard';

export const components = [
    {
        title: "Installation",
        icon: "📦",
        link: "/docs/getting-started/installation"
    },
    {
        title: 'Usage',
        icon: "🔨",
        link: "/docs/getting-started/usage"
    },
    {
        title: 'Comparison With Other Tools',
        icon: "⚖️",
        link: "/docs/getting-started/comparison-with-other-tools"
    },
    {
        title: 'Next Steps',
        icon: "➡️",
        link: "/docs/getting-started/next-steps"
    }
];

# 🚀 Getting Started with Ahnlich

<div className="remove-link-line grid xl:grid-cols-2 gap-4">
    {components.map((component) => (
        <CustomDocCard 
            key={component.title}
            title={component.title} 
            icon={component.icon} 
            link={component.link} 
        />
    ))}
</div>
