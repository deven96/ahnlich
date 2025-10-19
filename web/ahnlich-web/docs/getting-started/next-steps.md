---
title: ➡️ Next Steps
sidebar_position: 40
---

import CustomDocCard from '@site/src/components/CustomDocCard';

export const components = [
    {
        title: "Architecture",
        icon: "🏛️",
        link: "/docs/architecture",
        description: 'Learn about the architecture of Ahnlich'
    },
    {
        title: 'Guides',
        icon: "📘",
        link: "/docs/guides",
        description: 'Use cases and examples with Ahnlich'
    },
    {
        title: 'Community',
        icon: "🌍",
        link: "/docs/community",
        description: 'Get involved with Ahnlich'
    }
];

# Next steps

<div className="remove-link-line grid md:grid-cols-2 gap-4">
    {components.map((component) => (
        <CustomDocCard 
            key={component.title}
            title={component.title} 
            icon={component.icon} 
            link={component.link}
            description={component.description}
        />
    ))}
</div>