interface CodeBlockProps {
  code: string;
  language: string;
  onApply?: (code: string, type: 'ontology' | 'query') => void;
}

export function CodeBlock({ code, language, onApply }: CodeBlockProps) {
  const codeType = detectCodeType(code, language);

  const handleApply = () => {
    if (onApply && codeType) {
      onApply(code, codeType);
    }
  };

  return (
    <div className="code-block">
      <div className="code-block__header">
        <span>{language || 'code'}</span>
        {codeType && onApply && (
          <button className="code-block__apply" onClick={handleApply}>
            {codeType === 'ontology' ? 'Apply to Ontology' : 'Apply to Query'}
          </button>
        )}
      </div>
      <pre className="code-block__content">
        <code>{code}</code>
      </pre>
    </div>
  );
}

function detectCodeType(code: string, language: string): 'ontology' | 'query' | null {
  const lowerCode = code.toLowerCase();
  const lowerLang = language.toLowerCase();

  // Language-based detection
  if (lowerLang === 'mew' || lowerLang === 'ontology') {
    return 'ontology';
  }
  if (lowerLang === 'mew-gql' || lowerLang === 'gql' || lowerLang === 'query') {
    return 'query';
  }

  // Keyword-based detection for ontology
  const ontologyKeywords = ['ontology', 'node ', 'edge '];
  const hasOntologyKeyword = ontologyKeywords.some((kw) => lowerCode.includes(kw));

  // Keyword-based detection for query
  const queryKeywords = ['match', 'return', 'create', 'delete', 'insert', 'set '];
  const hasQueryKeyword = queryKeywords.some((kw) => lowerCode.includes(kw));

  // If it has ontology keywords and starts with ontology, it's an ontology
  if (hasOntologyKeyword && lowerCode.trimStart().startsWith('ontology')) {
    return 'ontology';
  }

  // If it has query keywords without ontology definition, it's a query
  if (hasQueryKeyword && !lowerCode.trimStart().startsWith('ontology')) {
    return 'query';
  }

  // Default: if it has ontology keyword, treat as ontology
  if (hasOntologyKeyword) {
    return 'ontology';
  }

  // Default: if it has query keyword, treat as query
  if (hasQueryKeyword) {
    return 'query';
  }

  return null;
}
