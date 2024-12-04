// components/CodeSnippet.js
import React from 'react';
import { Light as SyntaxHighlighter } from 'react-syntax-highlighter';
import { a11yDark } from 'react-syntax-highlighter/dist/esm/styles/hljs';
import yaml from 'react-syntax-highlighter/dist/esm/languages/hljs/yaml';

// Register YAML
SyntaxHighlighter.registerLanguage('yaml', yaml);

const CodeSnippet = ({ code, language = 'javascript' }: {code: string; language: string}) => {

  return (
    <div>
      <SyntaxHighlighter
        lineProps={{style: { whiteSpace: 'pre-wrap'}}}
        wrapLines={true}
        language={language} 
        style={a11yDark}
      >
        {code}
      </SyntaxHighlighter>
    </div>
  );
};

export default CodeSnippet;
