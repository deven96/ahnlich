import React from 'react';
import Giscus from '@giscus/react';
import { useColorMode } from '@docusaurus/theme-common';

export default function GiscusComments() {
  const { colorMode } = useColorMode();

  return (
    <div style={{ marginTop: '3rem' }}>
      <Giscus
        id="comments"
        repo="deven96/ahnlich"
        repoId="R_kgDOL1nJjA"
        category="General"
        categoryId="DIC_kwDOL1nJjM4C2ZsP"
        mapping="pathname"
        strict="0"
        reactionsEnabled="1"
        emitMetadata="0"
        inputPosition="top"
        theme={colorMode === 'dark' ? 'dark' : 'light'}
        lang="en"
        loading="lazy"
      />
    </div>
  );
}
