import { create } from 'zustand';

const SAMPLE_ONTOLOGY = `ontology TaskBoard {
  -- Type aliases for reusable constraints
  type Priority = Int [>= 1, <= 5]
  type Status = String [in: ["todo", "active", "done"]]
  type Role = String [in: ["owner", "member", "viewer"]]

  node User {
    name: String [required, length: 1..50],
    email: String [required, unique],
    active: Bool = true
  }

  node Project {
    name: String [required, length: 1..100],
    archived: Bool = false
  }

  node Task {
    title: String [required, length: 1..200],
    status: Status = "todo",
    priority: Priority = 3,
    due: Timestamp?
  }

  node Tag {
    name: String [required, unique, length: 1..30],
    color: String = "#6366f1"
  }

  -- Binary edges with cardinality
  edge owns(owner: User, project: Project)
  edge belongs_to(task: Task, project: Project) [task -> 1]
  edge assigned_to(task: Task, user: User) [task -> 0..1]

  -- Self-referential with constraints
  edge blocks(blocker: Task, blocked: Task) [no_self, acyclic]

  -- Symmetric edge (bidirectional friendship)
  edge collaborates(a: User, b: User) [symmetric]

  -- Edge with attributes
  edge tagged(task: Task, tag: Tag) {
    added_by: String?
  }

  -- K-ary edge: review connects reviewer, task, and outcome
  edge review(reviewer: User, task: Task, approver: User) {
    approved: Bool = false,
    comment: String?
  }
}`;

const SAMPLE_QUERY = `MATCH t: Task, p: Project,
      belongs_to(t, p)
RETURN p.name, t.title, t.status, t.priority
ORDER BY p.name, t.priority`;

const DEFAULT_SEED = `-- Users
SPAWN alice: User { name = "Alice", email = "alice@dev.io" }
SPAWN bob: User { name = "Bob", email = "bob@dev.io" }
SPAWN carol: User { name = "Carol", email = "carol@dev.io" }

-- Projects
SPAWN api: Project { name = "API v2" }
SPAWN web: Project { name = "Web App" }

-- Tasks for API project
SPAWN t1: Task { title = "Design endpoints", status = "done", priority = 1 }
SPAWN t2: Task { title = "Auth middleware", status = "active", priority = 2 }
SPAWN t3: Task { title = "Rate limiting", status = "todo", priority = 3 }

-- Tasks for Web project
SPAWN t4: Task { title = "Dashboard UI", status = "active", priority = 1 }
SPAWN t5: Task { title = "Settings page", status = "todo", priority = 4 }

-- Tags
SPAWN urgent: Tag { name = "urgent", color = "#ef4444" }
SPAWN backend: Tag { name = "backend", color = "#22c55e" }
SPAWN frontend: Tag { name = "frontend", color = "#3b82f6" }

-- Project ownership
LINK owns(#alice, #api)
LINK owns(#bob, #web)

-- Task assignments
LINK belongs_to(#t1, #api)
LINK belongs_to(#t2, #api)
LINK belongs_to(#t3, #api)
LINK belongs_to(#t4, #web)
LINK belongs_to(#t5, #web)

LINK assigned_to(#t2, #bob)
LINK assigned_to(#t4, #carol)

-- Dependencies
LINK blocks(#t1, #t2)
LINK blocks(#t2, #t3)

-- Collaboration
LINK collaborates(#alice, #bob)

-- Tags
LINK tagged(#t2, #urgent) { added_by = "alice" }
LINK tagged(#t2, #backend)
LINK tagged(#t4, #frontend)
LINK tagged(#t4, #urgent)

-- K-ary review edge
LINK review(#carol, #t1, #alice) { approved = true, comment = "LGTM" }`;

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
