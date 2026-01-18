import { create } from 'zustand';

const SAMPLE_ONTOLOGY = `ontology TodoApp {
  node User {
    name: String [required]
    email: String [required, unique]
  }

  node Task {
    title: String [required]
    status: String = "todo"
    priority: Int = 3
  }

  edge owns(user: User, task: Task)
  edge assigned_to(task: Task, user: User)
}`;

const SAMPLE_QUERY = 'MATCH t: Task RETURN t.title, t.status, t.priority';

const DEFAULT_SEED = `SPAWN u1: User { name = "Alice", email = "alice@example.com" }
SPAWN u2: User { name = "Bob", email = "bob@example.com" }
SPAWN t1: Task { title = "Design API", status = "done", priority = 1 }
SPAWN t2: Task { title = "Implement backend", status = "in_progress", priority = 2 }
SPAWN t3: Task { title = "Write tests", status = "todo", priority = 3 }
LINK owns(#u1, #t1)
LINK owns(#u1, #t2)
LINK owns(#u2, #t3)
LINK assigned_to(#t2, #u2)`;

interface EditorState {
  ontologyContent: string;
  queryContent: string;
}

interface EditorActions {
  setOntologyContent: (content: string) => void;
  setQueryContent: (content: string) => void;
}

export const useEditorStore = create<EditorState & EditorActions>((set) => ({
  ontologyContent: SAMPLE_ONTOLOGY,
  queryContent: SAMPLE_QUERY,

  setOntologyContent: (content) => set({ ontologyContent: content }),
  setQueryContent: (content) => set({ queryContent: content }),
}));

export { SAMPLE_ONTOLOGY, SAMPLE_QUERY, DEFAULT_SEED };
