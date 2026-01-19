```markdown
# MEW Visualization

**Version:** 1.0
**Status:** Capability
**Scope:** Observer interface, navigation, rendering, interaction

---

# Part I: Foundations

## 1.1 The Core Model

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   QUERY ──────▶ RESULTS ──────▶ RENDERER ──────▶ VISUAL + HITMAP    │
│                                                                      │
│   System           System           WASM            Bytes            │
│   concern          concern          module          (any format)     │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

The system executes queries and returns results. That's all.

A renderer transforms results into visual output. That's all.

The client displays visual output and handles interaction via hit map.

## 1.2 Separation of Concerns

| Layer | Responsibility | Knows About |
|-------|----------------|-------------|
| **System** | Execute queries, return results | Graph, types, time |
| **Renderer** | Transform results into visuals | Results, context |
| **Client** | Display visuals, handle interaction | Visual format, hit map |

The system knows nothing about visualization.
The renderer knows nothing about the client platform.
The client knows nothing about the graph structure.

---

# Part II: Query Results

## 2.1 StatementResult

Every statement execution returns:

```rust
pub enum StatementResult {
    Query(QueryResult),
    Mutation(MutationSummary),
    Mixed {
        mutations: MutationSummary,
        queries: QueryResult,
    },
    Transaction(TransactionResult),
    Empty,
}
```

## 2.2 QueryResult

```rust
pub struct QueryResult {
    pub columns: Vec<String>,     // Column names
    pub types: Vec<String>,       // Column types
    pub rows: Vec<Vec<Value>>,    // 2D array of values
}
```

## 2.3 Value

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Timestamp(i64),
    Duration(i64),
    NodeRef(NodeId),
    EdgeRef(EdgeId),
    List(Vec<Value>),
}
```

This is what renderers receive. Tabular data. Rows and columns.

---

# Part III: View

## 3.1 Definition

A View is a node in the graph:

```mew
node View {
  queries: List<String> [required]    -- one or more queries
  renderer: ID?                       -- reference to Renderer module
  renderer_config: Map<String, any>?  -- renderer-specific parameters
  time_cursor: Timestamp?             -- null = current time
  scope: ID?                          -- null = current scope
  name: String?
}
```

## 3.2 Multiple Queries

A View contains a list of queries. The renderer receives all results and interprets them as it chooses.

Common patterns:
- Single query: just render it
- Two queries: first is "main", second is "context" (what some call focal/peripheral)
- Multiple queries: comparison, aggregation, whatever the renderer supports

The system executes all queries. The renderer decides meaning.

## 3.3 Example

```mew
SPAWN v: View {
  queries = [
    "MATCH t: Task WHERE t.status = 'blocked' RETURN t, t.title, t.priority",
    "MATCH t: Task, p: Person, assigned_to(t, p) WHERE t.status = 'blocked' RETURN p"
  ],
  renderer = #graph_renderer,
  renderer_config = { layout: "force", show_labels: true }
}
```

---

# Part IV: Renderer

## 4.1 Contract

A renderer is a WASM module that exports:

```
render(input_ptr: *const u8, input_len: u32,
       output_ptr: *mut u8, output_cap: u32) -> i32
```

Input: serialized bytes.
Output: serialized bytes.
Returns: bytes written, or negative error code.

## 4.2 Input Format

```json
{
  "results": [
    {
      "columns": ["t", "title", "priority"],
      "types": ["NodeRef", "String", "Int"],
      "rows": [
        [{"NodeRef": "..."}, "Fix bug", 3],
        [{"NodeRef": "..."}, "Add feature", 2]
      ]
    },
    {
      "columns": ["p"],
      "types": ["NodeRef"],
      "rows": [
        [{"NodeRef": "..."}],
        [{"NodeRef": "..."}]
      ]
    }
  ],
  "context": {
    "viewport": { "width": 800, "height": 600 },
    "output_format": "svg",
    "selection": [
      { "result": 0, "row": 1 },
      { "entity": "#task_5" }
    ],
    "config": { "layout": "force", "show_labels": true }
  }
}
```

- `results`: array of QueryResult, one per query in the View
- `context.viewport`: render target dimensions
- `context.output_format`: which format to produce
- `context.selection`: what the client currently has selected
- `context.config`: renderer-specific parameters from View

## 4.3 Output Format

```json
{
  "format": "svg",
  "visual": "<svg>...</svg>",
  "hit_map": [
    {
      "bounds": { "type": "rect", "x": 0, "y": 0, "w": 100, "h": 50 },
      "ref": { "type": "row", "result": 0, "index": 0 }
    },
    {
      "bounds": { "type": "circle", "cx": 150, "cy": 25, "r": 20 },
      "ref": { "type": "entity", "id": "#person_1" }
    }
  ]
}
```

- `format`: what kind of data `visual` contains
- `visual`: the actual output (string for text formats, base64 for binary)
- `hit_map`: interaction regions

## 4.4 Output Formats

The renderer declares what formats it can produce. The client requests one.

| Format | Visual Content | Client Renders With |
|--------|----------------|---------------------|
| `svg` | SVG string | DOM or innerHTML |
| `html` | HTML string | innerHTML |
| `canvas2d` | Drawing command array | Canvas 2D API |
| `png` | Base64 image | `<img>` |
| `webgl` | Scene data | WebGL API |
| `gltf` | 3D model | Three.js |
| `pdf` | Base64 PDF | PDF viewer |
| `text` | Plain text | Pre element |

## 4.5 Renderer Metadata

```mew
node Renderer {
  name: String [required]
  output_formats: List<String> [required]
  config_schema: String?                    -- JSON Schema
  module: Bytes [required]                  -- WASM binary
}
```

---

# Part V: Hit Map

## 5.1 Purpose

The hit map is the universal interactivity layer. It maps regions of visual output back to data.

## 5.2 Structure

```
HitRegion {
  bounds: Bounds
  ref: Reference
}
```

### 5.2.1 Bounds

```
Bounds =
  | { type: "rect", x, y, w, h }
  | { type: "circle", cx, cy, r }
  | { type: "polygon", points: [[x, y], ...] }
  | { type: "path", d: "M0,0 L10,10..." }
```

### 5.2.2 Reference

```
Reference =
  | { type: "row", result: usize, index: usize }   -- row in specific result set
  | { type: "entity", id: NodeId | EdgeId }        -- graph entity
  | { type: "custom", data: any }                  -- renderer-defined
```

## 5.3 Client Usage

Client uses hit map to:
1. Determine what's under cursor/pointer
2. Build selection from user interaction
3. Pass selection back to renderer on next render

---

# Part VI: Selection

## 6.1 Client Responsibility

Selection is a client concept. The system doesn't know about it.

The client:
1. Tracks which items are selected
2. Passes selection to renderer in context
3. Receives visual output with selection rendered (however renderer chooses)

## 6.2 Selection Format

```json
{
  "selection": [
    { "result": 0, "row": 2 },
    { "result": 0, "row": 5 },
    { "entity": "#task_7" }
  ]
}
```

Selection can reference:
- Rows by result index and row index
- Entities by ID (extracted from NodeRef/EdgeRef values)

## 6.3 Selection Semantics

The renderer decides how to visualize selection:
- Highlight
- Border
- Color change
- Glow
- Anything

The client decides how selection is modified:
- Click to select
- Shift-click to add
- Drag to box-select
- Command input to select by pattern

---

# Part VII: Cockpit

## 7.1 Definition

The cockpit is the observer's workspace. It arranges panels.

```mew
node Cockpit {
  owner: ID [required]
  name: String?
}

edge has_panel(Cockpit, Panel) { order: Int }
```

## 7.2 Panel

A panel is a positioned container that renders a View.

```mew
node Panel {
  view: ID [required]
  position: Position
  size: Extent
}
```

Position and extent are render-context-dependent:

| Context | Position | Extent |
|---------|----------|--------|
| 2D | `{ x, y }` | `{ w, h }` |
| 3D | `{ x, y, z }` | `{ w, h, d }` |
| CLI | `{ buffer }` | `{ lines }` |

## 7.3 Cockpit State

```mew
edge cockpit_selection(Cockpit, any)      -- current selection
edge navigation_history(Cockpit, View) {
  timestamp: Timestamp
  panel: ID
}
edge actor_lens(Cockpit, any)             -- viewing as actor
```

---

# Part VIII: Navigation

## 8.1 Principle

Navigation is query modification. Changing what you see means changing the query.

## 8.2 Actions

| User Action | Effect |
|-------------|--------|
| Click item | Update View.queries to focus on it |
| Enter portal | Add scope to View, scope changes |
| Exit world | Remove scope from View |
| Filter | Add WHERE clause to query |
| Expand | Add relationship pattern to query |
| Back | Restore previous View from history |

## 8.3 History

Navigation creates history entries. Each entry is a View snapshot.

```mew
LINK navigation_history(#cockpit, #view_snapshot) {
  timestamp = now(),
  panel = #panel_1
}
```

Back/forward traverses history. History is just graph data.

---

# Part IX: Orientation

## 9.1 Purpose

The observer needs to know where they are:
- What types are in view
- What scope they're in
- What actor lens is active

## 9.2 Implementation

Orientation is a dedicated View with fixed queries:

```mew
SPAWN orientation: View {
  queries = [
    "META MATCH t: _NodeType RETURN t.name, COUNT(*)",
    "DESCRIBE SCOPE"
  ],
  renderer = #minimap_renderer
}
```

The client keeps orientation visible/accessible at all times.

---

# Part X: Actor Lens

## 10.1 Concept

Queries execute as an actor. Different actors see different results (per policy).

## 10.2 Switching

The client can switch the actor lens:

```mew
SET cockpit.actor_lens = #bob
```

Subsequent query execution happens as that actor. Same queries, different results.

## 10.3 Use Cases

- Debug permissions: see what a user sees
- Build empathy: understand different roles
- Test policy: verify access control

---

# Part XI: Standard Renderers

## 11.1 Library

MEW ships with default renderers:

| Renderer | Description |
|----------|-------------|
| `mew:render/auto` | Analyzes results, picks appropriate visualization |
| `mew:render/table` | Tabular display |
| `mew:render/graph` | Node-edge visualization |
| `mew:render/detail` | Single entity detail |
| `mew:render/text` | Plain text (CLI) |

These are ordinary WASM modules. Nothing privileged.

## 11.2 Auto-Selection

When View.renderer is null, the client picks based on:
- Result structure (single row vs many, has NodeRef/EdgeRef vs not)
- Available renderers
- Output format needed

---

# Part XII: Interaction Flow

## 12.1 Render Cycle

```
┌─────────────────────────────────────────────────────────────────────┐
│                                                                      │
│   1. Client has View (queries + renderer)                           │
│                                                                      │
│   2. Client executes all queries                                    │
│      → System returns List<StatementResult>                         │
│                                                                      │
│   3. Client builds renderer input                                   │
│      - Results (QueryResult from each)                              │
│      - Context (viewport, selection, config)                        │
│                                                                      │
│   4. Client invokes WASM renderer                                   │
│      → Renderer returns visual + hit_map                            │
│                                                                      │
│   5. Client displays visual according to format                     │
│                                                                      │
│   6. User interacts (click, hover, etc.)                            │
│      → Client uses hit_map to resolve target                        │
│      → Client updates selection or navigates                        │
│                                                                      │
│   7. If selection changed: re-render (step 3-5)                     │
│      If navigated: update View, re-execute (step 2-5)               │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## 12.2 Watch Mode

For live updates, use WATCH statements in View.queries:

```mew
SPAWN v: View {
  queries = [
    "WATCH t: Task WHERE t.status = 'blocked' [mode: watch]"
  ],
  renderer = #graph_renderer
}
```

System pushes results on change. Client re-renders.

---

# Part XIII: Gestures

## 13.1 Gesture Vocabulary

| Gesture | Typical Meaning |
|---------|-----------------|
| Click | Select (replace) |
| Shift-click | Select (add) |
| Double-click | Navigate into / expand |
| Right-click | Context menu |
| Drag | Move or link |
| Hover | Tooltip |

## 13.2 Gesture → Action

The client maps gestures to actions:

| Gesture + Target | Action |
|------------------|--------|
| Click + hit region | Select entity |
| Double-click + NodeRef | Navigate (update query to focus) |
| Double-click + portal | Enter world (update scope) |
| Drag + entity → entity | Link (mutation) |
| Drag + entity → trash | Delete (mutation) |

## 13.3 Command Input

Text input for queries and mutations:

```
> blocked tasks assigned to alice
> link $sel to #project_beta
> delete $sel
> as bob
```

`$sel` references current selection.

---

# Part XIV: Summary

## 14.1 Concept Hierarchy

| Concept | Definition |
|---------|------------|
| **View** | List of queries + renderer reference |
| **Renderer** | WASM module: results → visual + hit_map |
| **HitMap** | Bounds → Reference mapping for interaction |
| **Cockpit** | Observer's workspace, contains panels |
| **Panel** | Positioned container rendering a View |
| **Selection** | Client-tracked set of selected items |

## 14.2 Boundaries

| System Responsibility | Client Responsibility | Renderer Responsibility |
|-----------------------|-----------------------|-------------------------|
| Execute queries | Display visuals | Transform results |
| Return results | Handle interaction | Produce hit map |
| Enforce policy | Track selection | Visualize selection |
| Manage time | Manage navigation | Layout, styling |

## 14.3 Key Principles

| Principle | Meaning |
|-----------|---------|
| **Queries return data** | System doesn't know about visualization |
| **Renderers are WASM** | Portable, sandboxed, replaceable |
| **Hit map is universal** | Standard interactivity regardless of visual format |
| **Selection is client-side** | Passed to renderer, not stored in system |
| **Navigation is query change** | No separate navigation concept |
| **Everything is graph** | Views, Renderers, Cockpits are nodes |

---

*End of MEW Visualization Capability*
```
