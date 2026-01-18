import { MonacoEditor } from './MonacoEditor';
import { useEditorStore, useSessionStore, useChatStore, useUIStore } from '../../stores';

export function EditorPanel() {
  const ontologyContent = useEditorStore((state) => state.ontologyContent);
  const setOntologyContent = useEditorStore((state) => state.setOntologyContent);
  const loadOntology = useSessionStore((state) => state.loadOntology);
  const sendSilentRequest = useChatStore((state) => state.sendSilentRequest);
  const isGeneratingSeed = useUIStore((state) => state.isGeneratingSeed);
  const setIsGeneratingSeed = useUIStore((state) => state.setIsGeneratingSeed);

  const handleLoadOntology = () => {
    loadOntology(ontologyContent);
  };

  const handleGenerateSeed = async () => {
    setIsGeneratingSeed(true);
    try {
      await sendSilentRequest(
        'Generate seed data for the current ontology using SPAWN statements. Create 3-5 sample nodes for each node type with realistic example data.',
        { currentOntology: ontologyContent }
      );
    } finally {
      setIsGeneratingSeed(false);
    }
  };

  return (
    <div className="editor-panel">
      <div className="editor-panel__header">
        <span className="editor-panel__title">Ontology</span>
        <div className="editor-panel__actions">
          <button
            className="editor-panel__btn"
            onClick={handleLoadOntology}
          >
            Load Ontology
          </button>
          <button
            className="editor-panel__btn editor-panel__btn--generate"
            onClick={handleGenerateSeed}
            disabled={isGeneratingSeed}
          >
            {isGeneratingSeed ? (
              <>
                <span className="editor-panel__spinner" />
                Generating...
              </>
            ) : (
              'Generate Seed'
            )}
          </button>
        </div>
      </div>
      <div className="editor-panel__content">
        <MonacoEditor
          type="ontology"
          value={ontologyContent}
          onChange={setOntologyContent}
        />
      </div>
    </div>
  );
}
