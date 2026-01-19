# MEW Visualization Design Plan

**Version:** Draft 4 (Final)
**Status:** Ready for Review

---

## Revision History

| Draft | Changes |
|-------|---------|
| 1 → 2 | Separated Layout from Renderer; added Orientation, Gestures, Keyboard, Accessibility sections; clarified Client knowledge boundary; resolved module storage and scope contradictions |
| 2 → 3 | Defined all missing types; harmonized trigger values; added incremental layout support; made binary protocol default; added a11y defaults; refined migration plan with estimates |
| 3 → 4 | **Critical clarification:** Separated Control Plane (MEW kernel) from Presentation Plane (client patterns); View/Cockpit/Panel are client-side conventions, not core MEW concepts; replaced DrawCommand with direct canvas rendering (WASM draws to OffscreenCanvas) |

---

## 1. Executive Summary

This document describes the architecture for MEW's visualization system. The design separates concerns into:

- **Control Plane** (MEW kernel): Query execution, graph storage, policy enforcement, time management
- **Presentation Plane** (Client): Visualization, interaction, workspace management

**Critical principle:** The MEW kernel knows nothing about visualization. All visualization concepts (Views, Cockpits, Panels, Renderers) are **client-side patterns**, not core MEW features. Clients implement these patterns by creating nodes/edges in their own application ontology—the kernel just sees ordinary graph data.

---

## 1.1 Plane Separation

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         PRESENTATION PLANE                                   │
│                         (Client Concern)                                     │
│                                                                             │
│   Cockpit, Panel, View, Renderer, Selection, Navigation, Hit Maps          │
│   - Implemented by each client (web, CLI, mobile, etc.)                    │
│   - Patterns documented here as recommendations                             │
│   - Stored as ordinary nodes/edges in client's app ontology                │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                              BOUNDARY                                        │
│                     (Query API - the only interface)                        │
│                                                                             │
│   execute(queries, options) → StatementResult                               │
│   watch(queries, options) → Subscription                                    │
│                                                                             │
├─────────────────────────────────────────────────────────────────────────────┤
│                          CONTROL PLANE                                       │
│                          (MEW Kernel)                                        │
│                                                                             │
│   Graph storage, Type system, Query execution, Policy enforcement,          │
│   Time/versioning, Transactions, Scopes/Worlds                              │
│   - No knowledge of visualization                                           │
│   - No UI-specific types or behaviors                                       │
│   - Treats View/Cockpit/Panel as ordinary user-defined nodes               │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

**What this means:**

1. **No kernel changes for visualization** — MEW core is never modified for UI concerns
2. **Clients define their own ontology** — View, Cockpit, Panel schemas are client choices
3. **Patterns, not prescriptions** — This document recommends patterns; clients adapt as needed
4. **Portability** — Different clients (web, CLI, native) can implement different UI patterns while sharing the same kernel

---

## 2. Design Principles

### 2.1 Separation of Concerns

| Layer | Responsibility | Knowledge |
|-------|---------------|-----------|
| **System** | Execute queries, enforce policy, manage time | Graph structure, types, time |
| **Layout** | Compute spatial positions from results | Result structure, node/edge counts |
| **Renderer** | Generate visuals from positioned data | Layout, theme, selection |
| **Client** | Display, interact, orchestrate | References, visuals, hit regions |

**Clarification:** The Client understands References (opaque identifiers pointing to result rows or entities) but does not understand graph semantics (types, relationships, policies).

### 2.2 Everything is Graph (Client Choice)

Clients *may* store Views, Renderers, Cockpits, Panels as nodes in their app ontology. Navigation history and workspace state *may* be edges. This makes UI state queryable and persistent—but it's a client pattern, not a kernel requirement.

A minimal client could keep all UI state in memory and never persist it. A sophisticated client could store everything in the graph. Both are valid.

### 2.3 Format Agnostic

Renderers declare supported output formats. Clients request one. The same data can produce SVG, Canvas commands, or text.

### 2.4 Portable and Sandboxed

Renderers are WASM modules with strict resource limits. They run in browser, server, or CLI without platform dependencies.

### 2.5 Adaptive by Default

The default renderer (`mew:render/auto`) adapts to any query result shape—graph, table, detail, or mixed. Clients don't need selection logic; they just render. Specialized renderers exist for specific needs but aren't required.

---

## 3. Presentation Entities (Client Patterns)

> **Note:** The entities in this section are **recommended patterns**, not MEW kernel types. Clients create these as ordinary nodes/edges in their application ontology. The kernel sees them as regular graph data with no special semantics.
>
> A CLI client might implement a minimal View concept with just queries. A web client might implement full Cockpit/Panel hierarchy. A mobile client might use entirely different patterns. All are valid—the kernel doesn't care.

### 3.1 View (Pattern)

A View is a named perspective on the graph. Clients typically store Views as nodes:

```
View {
  queries: List<String> [required]
  renderer: ID?                       -- null = client auto-selects
  layout: ID?                         -- null = renderer default
  renderer_config: Map?
  time_cursor: Timestamp?             -- null = current time
  scope: ID?                          -- null = inherit from context
  name: String?
}
```

**Scope inheritance:** When `scope` is null, inherit from Cockpit scope, then Session scope, then root.

### 3.2 Layout (Pattern)

A Layout is a spatial positioning algorithm. Clients may store Layout definitions as nodes:

```
Layout {
  name: String [required]
  algorithm: "force" | "hierarchical" | "radial" | "grid" | "custom"
  module_uri: String?                 -- for custom algorithms
  config_schema: String?              -- JSON Schema
  supports_incremental: Bool          -- can handle delta updates
}
```

**Standard layouts:**
- `force`: Force-directed (cyclic graphs) — supports incremental
- `hierarchical`: Tree/DAG (top-down or left-right)
- `radial`: Circular around focal node
- `grid`: Regular grid placement

### 3.3 Renderer (Pattern)

A Renderer transforms positioned data into visual output. Clients may register Renderers as nodes:

```
Renderer {
  name: String [required]
  input_format: "msgpack" | "json"    -- msgpack is default
  output_formats: List<String> [required]   -- ["canvas", "svg", "text"]
  config_schema: String?
  theme_extensions: Map?
  module_uri: String [required]
  module_hash: String?
  version: String [required]
}
```

**Output formats:**
- `canvas` — Direct mode: WASM draws to OffscreenCanvas (best performance)
- `svg` — Portable mode: returns SVG string (for export, SSR)
- `text` — Portable mode: returns plain text (for CLI)

**Module loading:** Primary via `module_uri`. For offline scenarios, bytes can be stored in a linked `RendererModule` node.

### 3.4 Cockpit (Pattern)

A Cockpit is an observer's workspace. This is a recommended pattern for organizing multiple views:

```
Cockpit {
  owner: ID [required]
  name: String?
  layout_mode: "free" | "grid" | "stack"
  scope: ID?                          -- default scope for panels
}

has_panel(Cockpit, Panel) {
  order: Int
  region: String?
}
```

### 3.5 Panel (Pattern)

A Panel is a positioned container that renders a View:

```
Panel {
  view: ID [required]
  position: Position
  size: Extent
  title: String?
  collapsed: Bool
}
```

### 3.6 Geometry Types

```
Position {
  x: Float
  y: Float
  z: Float?                           -- for 3D contexts
}

Extent {
  width: Float
  height: Float
  depth: Float?                       -- for 3D contexts
}

Bounds {
  type: "rect" | "circle" | "polygon"
  -- For rect:
  x: Float, y: Float, w: Float, h: Float
  -- For circle:
  cx: Float, cy: Float, r: Float
  -- For polygon:
  points: List<[Float, Float]>
}
```

### 3.7 Cockpit State Edges (Pattern)

Clients may persist workspace state as edges. These are optional patterns:

```
cockpit_selection(Cockpit, Reference) {
  timestamp: Timestamp
}

navigation_history(Cockpit, View) {
  timestamp: Timestamp
  panel: ID
  trigger: "click" | "query" | "scope" | "time" | "manual"
}

actor_lens(Cockpit, Actor) {
  timestamp: Timestamp
}

focus_state(Cockpit) {
  panel: ID?
}
```

---

## 4. Data Flow

### 4.1 Pipeline Overview

```
View.queries ──▶ SYSTEM ──▶ Results ──▶ LAYOUT ──▶ Positioned ──▶ RENDERER ──▶ Visual + HitMap
                execute                  compute     data          render
                                           ▲                          │
                                           │                          ▼
                                    (cache)                       CLIENT
                                                              display, interact
```

### 4.2 Result Types

System returns `StatementResult`. Visualization uses the `QueryResult` variant:

```
StatementResult =
  | Query(QueryResult)
  | Mutation(MutationSummary)
  | Mixed { mutations, queries }
  | Transaction(...)
  | Empty

QueryResult {
  columns: List<String>
  types: List<String>
  rows: List<List<Value>>
  version: Int                        -- for cache invalidation
}
```

Layout and Renderer receive `QueryResult`, extracted from `StatementResult`.

### 4.3 Layout Computation

**Input:**
```
LayoutInput {
  results: List<QueryResult>
  algorithm: String
  config: Map
  constraints: LayoutConstraints
  previous: LayoutOutput?             -- for incremental updates
}

LayoutConstraints {
  focal: List<Reference>              -- nodes to emphasize
  pinned: List<PinnedNode>            -- fixed positions
  viewport: Extent                    -- target size
}

PinnedNode {
  ref: Reference
  position: Position
}
```

**Output:**
```
LayoutOutput {
  positions: List<NodePosition>
  edges: List<EdgePath>
  bounds: Bounds
  version: Int                        -- for cache key
}

NodePosition {
  ref: Reference
  x: Float
  y: Float
  width: Float
  height: Float
}

EdgePath {
  from: Reference
  to: Reference
  path: List<[Float, Float]>
}
```

**Caching:** Cache key = `hash(results.version, algorithm, config, constraints)`. Use version ID, not content hash.

**Incremental updates:** If `supports_incremental` and `previous` provided, layout engine preserves existing positions and only computes new/removed nodes.

### 4.4 Renderer Contract

Renderers support two modes: **direct** (canvas) and **portable** (svg, text).

**Input (MessagePack by default):**
```
RenderInput {
  layout: LayoutOutput
  results: List<QueryResult>
  context: RenderContext
}

RenderContext {
  viewport: Extent
  output_format: "canvas" | "svg" | "text"
  selection: List<Reference>
  hover: Reference?
  focus: Reference?
  config: Map
  theme: Theme
}
```

### 4.5 Output Formats

| Format | Mode | How it works |
|--------|------|--------------|
| `canvas` | Direct | WASM receives OffscreenCanvas, draws directly, returns HitMap + a11y only |
| `svg` | Portable | WASM returns SVG string + HitMap + a11y |
| `text` | Portable | WASM returns plain text string + HitMap + a11y |

**Default:** `canvas` (direct rendering, no serialization overhead).

### 4.6 Direct Mode (canvas)

```
// Client passes OffscreenCanvas to WASM
render_canvas(
  canvas: OffscreenCanvas,
  input: RenderInput
) -> DirectOutput

DirectOutput {
  hit_map: HitMap
  a11y: AccessibilityTree
  metadata: RenderMetadata
}
```

WASM draws directly to the canvas using the 2D context. No DrawCommands, no serialization — maximum performance.

**Why OffscreenCanvas:**
- Can be transferred to WASM
- Works in main thread or Web Worker
- Full Canvas 2D API available

### 4.7 Portable Mode (svg, text)

```
render_portable(input: RenderInput) -> PortableOutput

PortableOutput {
  format: "svg" | "text"
  content: String                     -- SVG markup or plain text
  hit_map: HitMap
  a11y: AccessibilityTree
  metadata: RenderMetadata
}
```

Use portable mode for:
- Export to file (SVG, PDF via svg-to-pdf)
- CLI clients (text)
- Server-side rendering
- Debugging (inspect SVG output)

### 4.8 Render Metadata

```
RenderMetadata {
  bounds: Bounds
  focal_bounds: Bounds?
  warnings: List<String>
}
```

---

## 5. Hit Map

### 5.1 Structure

```
HitMap {
  regions: List<HitRegion>
  index: SpatialIndex?                -- implementation detail (quadtree)
}
```

### 5.2 HitRegion

```
HitRegion {
  bounds: Bounds
  ref: Reference
  layer: Int                          -- higher = on top
  cursor: "default" | "pointer" | "grab" | "text"
  tooltip: TooltipContent?
  affordance: "clickable" | "draggable" | "expandable"?
  actions: List<ActionHint>?
}
```

### 5.3 Supporting Types

```
TooltipContent {
  title: String
  subtitle: String?
  details: List<{ label: String, value: String }>?
  shortcuts: List<{ key: String, action: String }>?
}

ActionHint {
  id: String                          -- "delete", "expand", "link"
  label: String                       -- "Delete node"
  shortcut: String?                   -- "⌫"
  destructive: Bool
}
```

### 5.4 Reference Types

```
Reference =
  | { type: "row", result: Int, index: Int }
  | { type: "entity", id: String }
  | { type: "cell", result: Int, row: Int, col: Int }
  | { type: "edge", from: Reference, to: Reference }
  | { type: "custom", kind: String, data: Any }
```

### 5.5 Hit Testing

Client performs hit testing using spatial index:

```typescript
function hitTest(hitMap: HitMap, x: number, y: number): HitRegion | null {
  const candidates = hitMap.index.query(x, y);
  candidates.sort((a, b) => b.layer - a.layer);  // highest layer first
  for (const region of candidates) {
    if (containsPoint(region.bounds, x, y)) {
      return region;
    }
  }
  return null;
}
```

---

## 6. Accessibility

### 6.1 Contract

Accessibility is **required**. Every renderer must provide an `AccessibilityTree`:

```
AccessibilityTree {
  description: String                 -- overall visualization description
  items: List<A11yItem>
  navigation_order: List<Reference>   -- default: spatial order (left-to-right, top-to-bottom)
  landmarks: List<A11yLandmark>
  live_announcements: List<String>
}

A11yItem {
  ref: Reference
  label: String                       -- "Person: Alice Chen"
  role: "button" | "link" | "listitem" | "treeitem"
  description: String?
  state: A11yState
}

A11yState {
  selected: Bool
  expanded: Bool?
  level: Int?                         -- for tree structures
}

A11yLandmark {
  label: String
  ref: Reference
}
```

### 6.2 Default Generator

For renderers that don't provide custom a11y, Client generates a default tree from Layout:
- Each positioned node becomes an A11yItem
- Navigation order = spatial order (top-to-bottom, left-to-right)
- Label = node type + first string attribute

### 6.3 Client Responsibilities

1. Render a11y tree as hidden DOM or ARIA overlay
2. Manage focus per `navigation_order`
3. Announce `live_announcements` via ARIA live region (debounced for watch mode)
4. Sync visual focus indicator with a11y focus

### 6.4 Contrast Requirements

Themes must meet WCAG 2.1 AA:
- Text contrast ratio ≥ 4.5:1
- Large text (≥18pt) contrast ratio ≥ 3:1
- UI component contrast ratio ≥ 3:1

---

## 7. Keyboard Navigation

### 7.1 Global Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Next panel |
| `Shift+Tab` | Previous panel |
| `Escape` | Clear selection / close modal |
| `?` | Show keyboard help |
| `/` | Focus command input |

### 7.2 Within Panel

| Key | Action |
|-----|--------|
| `Arrow keys` | Navigate between items |
| `Enter` / `Space` | Activate focused item |
| `Shift+Arrow` | Extend selection |
| `Cmd/Ctrl+A` | Select all |
| `Cmd/Ctrl+D` | Deselect all |
| `Home` / `End` | First / last item |

### 7.3 Navigation

| Key | Action |
|-----|--------|
| `Cmd/Ctrl+[` | Back in history |
| `Cmd/Ctrl+]` | Forward in history |
| `Enter` on entity | Navigate into |
| `Backspace` | Exit scope |

### 7.4 Focus State

```
FocusState {
  panel: Panel?
  item: Reference?
  mode: "navigate" | "select"
}
```

---

## 8. Gestures

### 8.1 Vocabulary

| Gesture | Meaning |
|---------|---------|
| Click | Select (replace) |
| Shift+Click | Add to selection |
| Cmd/Ctrl+Click | Toggle in selection |
| Double-click | Navigate into / expand |
| Right-click | Context menu |
| Drag (empty) | Pan viewport |
| Drag (item) | Move item |
| Wheel | Zoom |
| Pinch | Zoom (touch) |
| Long-press | Context menu (touch) |

### 8.2 Action Mapping

| Gesture + Target | Action |
|------------------|--------|
| Click + hit region | Select |
| Double-click + NodeRef | Navigate (focus query) |
| Double-click + scope | Enter world |
| Drag + entity → entity | Create link |
| Drag + entity → trash | Delete |
| Right-click + entity | Context menu |

### 8.3 Touch Fallbacks

| Touch | Maps To |
|-------|---------|
| Tap | Click |
| Double-tap | Double-click |
| Two-finger tap | Right-click |
| Long-press | Right-click |
| Pan | Drag (empty) |
| Pinch | Wheel |

---

## 9. Selection

### 9.1 State

```
Selection {
  items: List<Reference>
  anchor: Reference?                  -- for range selection
  mode: "single" | "multi"
}
```

### 9.2 Semantics

- Client owns selection state
- Passed to renderer in context
- Renderer decides visual treatment
- Optionally persisted to `cockpit_selection` edge

### 9.3 Policy

```
SelectionPolicy {
  multi_select: Bool
  cross_panel: Bool
  allowed_types: List<String>?
}
```

---

## 10. Navigation

### 10.1 Principle

Navigation is query modification. Changing what you see means changing the View.

### 10.2 Actions

| Action | View Change |
|--------|-------------|
| Click entity | Add focus filter |
| Enter portal | Set View.scope |
| Exit world | Clear View.scope |
| Filter | Add WHERE clause |
| Expand | Add relationship pattern |
| Time travel | Set View.time_cursor |
| Back/Forward | Restore from history |

### 10.3 History

```
NavigationHistory {
  entries: List<ViewSnapshot>
  current_index: Int
}

ViewSnapshot {
  view: View
  timestamp: Timestamp
  trigger: "click" | "query" | "scope" | "time" | "manual"
  panel: Panel
}
```

**Constraints:**
- Linear stack (not tree)
- Forward history cleared on new navigation
- Max 100 entries per cockpit

### 10.4 Breadcrumbs

```
[Root] > [Project Alpha] > [Task: Fix Bug]
       scope               focal entity
```

### 10.5 Deep Linking

```
/cockpit/:id?view=:viewId&selection=:refs&time=:cursor
```

---

## 11. Orientation

### 11.1 Purpose

Observer awareness of:
- Types in view (counts)
- Current scope path
- Actor lens status
- History position

### 11.2 Implementation

Dedicated View with fixed queries:

```
orientation_view: View {
  queries = [
    "META MATCH t: _NodeType RETURN t.name, COUNT(*)",
    "DESCRIBE SCOPE"
  ],
  renderer = #minimap_renderer
}
```

---

## 12. Feedback States

### 12.1 Loading State

```
LoadingState =
  | { type: "idle" }
  | { type: "loading", operation: String, started: Timestamp }
  | { type: "progress", operation: String, percent: Float }
  | { type: "error", message: String, retry: Function }
```

### 12.2 Timing

| Operation | Show feedback after |
|-----------|---------------------|
| Query execution | 200ms |
| Layout computation | 500ms |
| WASM load | 0ms (show immediately) |
| Network fetch | 200ms |

### 12.3 Transitions

| Change | Transition |
|--------|------------|
| Selection | Instant |
| Navigation | Fade 150ms |
| Layout change | Animate 300ms |
| Zoom/pan | requestAnimationFrame |

---

## 13. Theming

### 13.1 Structure

```
Theme {
  colors: {
    background, surface, border,
    text_primary, text_secondary,
    accent, success, warning, error,
    node_default, node_focal, node_selected,
    edge_default, edge_focal
  },
  typography: {
    font_family, font_size_base, line_height
  },
  spacing: {
    node_padding, node_gap, edge_spacing
  },
  shapes: {
    node_radius, edge_width, selection_width
  }
}
```

### 13.2 Extensions

Renderers declare additional keys with defaults:

```
Renderer {
  theme_extensions: {
    "chart_colors": ["#6366f1", "#22c55e"],
    "grid_line_color": "#333333"
  }
}
```

---

## 14. Actor Lens

### 14.1 Concept

Queries execute as an actor. Different actors see different results per policy.

### 14.2 Usage

```mew
SET cockpit.actor_lens = #bob
```

### 14.3 Indicator

When active:
- Show "Viewing as: Bob" in orientation
- Optionally dim elements Bob can't access

---

## 15. System API

### 15.1 Query Execution

```
execute(
  queries: List<String>,
  options: ExecuteOptions
) -> List<StatementResult>

ExecuteOptions {
  scope: ID?
  time_cursor: Timestamp?
  actor: ID?
  timeout: Duration?
}
```

### 15.2 Watch Mode

```
watch(
  queries: List<String>,
  options: WatchOptions
) -> Subscription

Subscription {
  on_result(results: List<StatementResult>)
  on_delta(delta: ResultDelta)
  on_error(error: Error)
  cancel()
}

ResultDelta {
  type: "insert" | "update" | "delete"
  result_index: Int
  row_index: Int
  row: Row?
}
```

**Delta routing:** Deltas trigger incremental layout update if layout `supports_incremental`, otherwise full recompute.

### 15.3 Layout Service

```
compute_layout(
  input: LayoutInput
) -> LayoutOutput
```

### 15.4 Renderer Registry

```
list_renderers(filter?: { formats?: List<String> }) -> List<Renderer>
get_renderer(id: ID) -> Renderer
load_renderer_module(uri: String) -> WasmModule
```

---

## 16. Standard Components

### 16.1 Layouts

| Layout | Incremental | Use Case |
|--------|-------------|----------|
| `mew:layout/force` | Yes | General graphs, cyclic |
| `mew:layout/hierarchical` | No | Trees, DAGs |
| `mew:layout/radial` | No | Focal node with context |
| `mew:layout/grid` | Yes | Uniform collections |

### 16.2 Renderers

| Renderer | Formats | Use Case |
|----------|---------|----------|
| `mew:render/auto` | canvas, svg, text | **Default.** Adapts to any result shape |
| `mew:render/graph` | canvas, svg | Node-edge visualization (specialized) |
| `mew:render/table` | canvas, svg, text | Tabular data (specialized) |
| `mew:render/detail` | canvas, svg, text | Single entity focus (specialized) |
| `mew:render/text` | text | CLI output |

### 16.3 Default Renderer: `mew:render/auto`

The default renderer is a single adaptive WASM module that handles any query result shape. When `View.renderer` is null, clients use this.

**Behavior:**

| Result Shape | Rendering Strategy |
|--------------|-------------------|
| Single row, has NodeRef/EdgeRef | Detail view: entity card with attributes, related items radially |
| Multiple rows, has NodeRef/EdgeRef | Graph view: nodes positioned by layout, edges drawn |
| Multiple rows, no refs | Table view: columns as headers, rows as cells |
| Single row, no refs | Key-value view: vertical attribute list |
| Empty results | Empty state with message |
| Mixed (multiple queries) | Split view: each result set rendered appropriately |

**Why one adaptive renderer instead of delegation:**
- Single WASM module load (faster startup)
- Consistent visual language across modes
- Smooth transitions when result shape changes (e.g., filter narrows to single entity)
- Simpler client logic (always use auto, let it figure it out)

**Implementation sketch:**

```rust
fn render(canvas: &OffscreenCanvas, input: &RenderInput) -> DirectOutput {
    let strategy = analyze_results(&input.results);

    match strategy {
        Strategy::Detail(entity) => render_detail(canvas, entity, input),
        Strategy::Graph(nodes, edges) => render_graph(canvas, nodes, edges, input),
        Strategy::Table(columns, rows) => render_table(canvas, columns, rows, input),
        Strategy::KeyValue(pairs) => render_key_value(canvas, pairs, input),
        Strategy::Empty => render_empty_state(canvas, input),
        Strategy::Mixed(parts) => render_split(canvas, parts, input),
    }
}
```

**Specialized renderers still exist** for cases where you want specific behavior (e.g., force-directed graph with custom physics, or a table with specific column formatting). But for most cases, `mew:render/auto` just works.

### 16.4 Renderer Selection

```
If View.renderer is specified → use it
Else → use mew:render/auto (default)
```

That's it. No client-side heuristics needed.

---

## 17. Error Handling

### 17.1 Categories

| Category | Handling |
|----------|----------|
| Query syntax error | Show in editor |
| Query execution error | Show in panel, allow retry |
| Layout error | Fall back to grid |
| Renderer error | Fall back to table → text → raw JSON |
| WASM panic | Reload module |
| Network error | Show offline, retry with backoff |

### 17.2 Partial Results

- Render successful queries
- Show errors inline for failures
- Don't block entire view

---

## 18. Performance

### 18.1 Targets

| Scenario | Target |
|----------|--------|
| <50 nodes | 60fps |
| 50-500 nodes | 30fps |
| >500 nodes | Progressive render |

### 18.2 Rendering Modes

| Mode | Serialization | Overhead | Use case |
|------|---------------|----------|----------|
| `canvas` (direct) | Input only (msgpack) | Minimal | Interactive (default) |
| `svg` (portable) | Input + output | Higher | Export, SSR |
| `text` (portable) | Input + output | Higher | CLI |

**Direct canvas mode eliminates output serialization entirely.** WASM draws directly to OffscreenCanvas — only the HitMap and a11y tree are returned.

Input serialization overhead (MessagePack):
| Graph size | Input serialization |
|------------|---------------------|
| 100 nodes | ~0.5ms |
| 500 nodes | ~2ms |

### 18.3 Caching

| Data | Cache Key | Invalidation |
|------|-----------|--------------|
| Results | query + options hash | TTL or watch |
| Layout | results.version + algorithm + config | Results change |
| Render | layout.version + selection + theme | Selection change |
| WASM | URI + version | Never |

### 18.4 Large Results

1. Query: LIMIT/OFFSET
2. Layout: Viewport culling
3. Render: Virtual scroll / LOD
4. Client: Throttle re-renders

---

## 19. Security

### 19.1 Sandboxing

WASM modules run with:
- Memory: 256MB max
- Time: 5s per render
- No network
- No filesystem
- Imports: math only

### 19.2 Trust Levels

| Source | Trust |
|--------|-------|
| `mew:*` | Full |
| Verified publisher | User-approved |
| Unknown | Strict limits, user prompt |

### 19.3 Validation

- Input validated against schema
- SVG output sanitized (no scripts)

---

## 20. Extension Points

### 20.1 Custom Renderers

1. Implement WASM interface
2. Register Renderer node
3. Reference from View.renderer

### 20.2 Custom Layouts

1. Implement layout WASM interface
2. Register Layout node
3. Reference from View.layout

### 20.3 Custom Gestures

```typescript
cockpit.onGesture((gesture, target) => {
  if (gesture.type === "drag" && target.ref.type === "custom") {
    // Custom handling
    return true;  // consumed
  }
  return false;  // default handling
});
```

### 20.4 Renderer Composition

```
{ "op": "subrender", "renderer": "mew:render/table", "input": {...},
  "x": 0, "y": 0, "width": 400, "height": 300 }
```

Client resolves recursively. Hit maps merged with coordinate transform.

---

## 21. Migration Path

> **Scope:** This migration is entirely within the **Presentation Plane**. No MEW kernel changes are required or planned. The kernel continues to execute queries and return results—it doesn't know or care about visualization.

### Phase 0: Infrastructure (1 week)

- Feature flags for old/new renderer
- Visual regression test setup
- A/B switching capability

### Phase 1: Interfaces (2 weeks)

- WASM interface specification
- JSON/MessagePack schemas
- Hit map format specification

### Phase 2: Layout Extraction (2 weeks)

- Extract algorithms from current renderer
- Implement Layout WASM module
- Add caching layer
- Implement incremental update for force layout

### Phase 3: First Renderers (4 weeks)

- Graph renderer (port from Canvas)
- Table renderer
- Text renderer
- Default a11y generator

### Phase 4: Client Refactor (3 weeks)

- Render orchestration in React
- Hit map interaction
- Selection/navigation state management
- Keyboard navigation

### Phase 5: Client Ontology & Persistence (3 weeks)

- Define View/Renderer/Cockpit types in client's app ontology
- Persist navigation history as graph edges (optional pattern)
- Deep linking / URL state sync

### Phase 6: Polish (2 weeks)

- Accessibility audit (VoiceOver, NVDA)
- Performance profiling
- Documentation

**Total estimate:** 17 weeks

**Critical path:** Phase 1 → 2 → 3 → 4 (blocking)
**Parallelizable:** Phase 5 can start after Phase 1

---

## 22. Open Questions

1. **Collaboration:** Real-time multi-user editing of shared cockpits?
2. **Offline:** Service worker strategy for client assets?
3. **Mobile:** Native vs PWA for mobile clients?
4. **Marketplace:** Renderer distribution/discovery mechanism?

---

## 23. Summary: What This Document Is and Isn't

**This document IS:**
- A design for the **Presentation Plane** (client-side visualization)
- A set of **recommended patterns** for UI state management
- A specification for **WASM renderer interfaces**
- A guide for building MEW visualization clients

**This document is NOT:**
- A specification for MEW kernel changes
- A requirement that all clients implement these patterns
- A modification to MEW's core ontology or query semantics

The MEW kernel remains unchanged. It executes queries and returns `StatementResult`. Everything else—Views, Cockpits, Panels, Renderers, hit maps, selection, navigation—is client concern.

---

*End of Draft 4*
