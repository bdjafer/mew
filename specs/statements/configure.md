---
spec: configure
version: "1.0"
status: draft
category: statement
capability: time
requires: [expression]
priority: common
---

# Spec: CONFIGURE

## Overview

CONFIGURE statements control MEW's timing and execution model. They allow loading preset configurations, setting individual configuration values, and inspecting current settings. These statements enable applications to tune MEW for different use cases: traditional databases, real-time games, deterministic simulations, or high-throughput streaming.

## Syntax

### Grammar

```ebnf
ConfigStmt       = PresetStmt | SetStmt | ShowStmt ;

PresetStmt       = "CONFIGURE" "PRESET" PresetName ;

PresetName       = "database" | "game" | "simulation" | "streaming" ;

SetStmt          = "SET" ConfigPath "=" ConfigValue ;

ConfigPath       = Identifier ("." Identifier)* ;

ConfigValue      = StringLiteral
                 | IntLiteral
                 | Duration
                 | BoolLiteral ;

Duration         = IntLiteral ("ms" | "s" | "m" | "h") ;

BoolLiteral      = "true" | "false" ;

ShowStmt         = "SHOW" "CONFIGURATION" ConfigPath? ;
```

### Keywords

| Keyword | Context |
|---------|---------|
| `CONFIGURE` | Statement - initiates preset loading |
| `PRESET` | Clause - specifies preset to load |
| `database` | Preset name - traditional database behavior |
| `game` | Preset name - fixed timestep game loop |
| `simulation` | Preset name - deterministic manual control |
| `streaming` | Preset name - high-throughput event processing |
| `SET` | Statement - sets a configuration value |
| `SHOW` | Statement - displays configuration |
| `CONFIGURATION` | Clause - specifies configuration display |
| `true` | Boolean value - enables option |
| `false` | Boolean value - disables option |
| `ms` | Duration unit - milliseconds |
| `s` | Duration unit - seconds |
| `m` | Duration unit - minutes |
| `h` | Duration unit - hours |

### Examples

```
-- Load a preset configuration
CONFIGURE PRESET game

-- Set individual configuration values
SET time.now_source = "logical"
SET time.tick_interval = 16ms
SET execution.max_rule_depth = 200

-- Show current configuration
SHOW CONFIGURATION
SHOW CONFIGURATION time
SHOW CONFIGURATION execution.rule_mode
```

## Semantics

### CONFIGURE PRESET

Loads a named configuration preset, setting multiple configuration values atomically.

When CONFIGURE PRESET executes:
1. All timing configuration values are set according to the preset
2. All execution configuration values are set according to the preset
3. Previous configuration is completely replaced (not merged)

Presets provide consistent configurations for common use cases. After loading a preset, individual values can be overridden with SET.

See also: The TICK statement advances logical time and is controlled by `time.tick_on` configuration.

### SET

Sets a single configuration value at the specified path.

When SET executes:
1. The configuration path is validated
2. The value type is checked against the expected type
3. The value is validated against allowed options or ranges
4. The configuration is updated atomically

SET statements take effect immediately. Running configurations affect all subsequent operations in the session and any transactions it creates.

### SHOW CONFIGURATION

Displays current configuration values.

| Form | Behavior |
|------|----------|
| `SHOW CONFIGURATION` | Display all configuration |
| `SHOW CONFIGURATION time` | Display timing configuration |
| `SHOW CONFIGURATION execution` | Display execution configuration |
| `SHOW CONFIGURATION time.now_source` | Display single value |

The output format is implementation-defined but includes the path and current value for each setting.

## Configuration Paths

### Timing Configuration (time.*)

| Path | Type | Options | Default | Description |
|------|------|---------|---------|-------------|
| `time.now_source` | String | "wall", "logical", "hybrid" | "wall" | What `now()` returns |
| `time.tick_on` | String | "manual", "commit", "interval" | "commit" | When ticks occur |
| `time.tick_interval` | Duration | - | none | Interval for periodic ticks |

**now_source options:**
- `"wall"` - `now()` returns `wall_time()` (real-world clock)
- `"logical"` - `now()` returns `logical_time()` (tick counter)
- `"hybrid"` - `now()` returns tuple of `(wall_time(), logical_time())`

**tick_on options:**
- `"manual"` - ticks only via explicit `TICK` statement
- `"commit"` - tick advances on each transaction commit
- `"interval"` - tick advances at `tick_interval` wall-clock intervals

### Execution Configuration (execution.*)

| Path | Type | Options | Default | Description |
|------|------|---------|---------|-------------|
| `execution.input_mode` | String | "immediate", "queued", "batched" | "immediate" | How mutations enter |
| `execution.rule_mode` | String | "eager", "deferred", "explicit" | "eager" | When rules fire |
| `execution.quiescence` | String | "strict", "bounded", "best_effort" | "strict" | Quiescence handling |
| `execution.on_limit` | String | "rollback", "commit", "defer" | "rollback" | Behavior on limit |
| `execution.deterministic` | Bool | true, false | false | Enforce determinism |
| `execution.max_rule_depth` | Int | - | 100 | Max nested rule depth |
| `execution.max_rule_actions` | Int | - | 10000 | Max actions per tick |
| `execution.max_execution_time` | Duration | - | 1000ms | Wall-clock budget |
| `execution.max_queue_size` | Int | - | 10000 | Max pending mutations |
| `execution.max_batch_size` | Int | - | 1000 | Max mutations per batch |
| `execution.time_budget` | Duration | - | none | Time budget for best_effort |
| `execution.queue_capacity` | Int | - | 10000 | Mutation queue capacity |
| `execution.batch_window` | Duration | - | none | Batch accumulation window |

**input_mode options:**
- `"immediate"` - mutations processed synchronously, client blocks
- `"queued"` - mutations added to queue, acknowledged, processed later
- `"batched"` - mutations accumulate until tick, then processed together

**rule_mode options:**
- `"eager"` - rules fire immediately after each mutation
- `"deferred"` - rules fire on next tick
- `"explicit"` - rules only fire via explicit `PROCESS RULES` statement

**quiescence options:**
- `"strict"` - wait for full quiescence; rollback if limits hit
- `"bounded"` - process up to limit, commit partial state
- `"best_effort"` - process until time budget exhausted, commit current state

**on_limit options:**
- `"rollback"` - abort transaction if limit exceeded
- `"commit"` - commit partial state
- `"defer"` - commit partial, continue later

### Authorization Configuration (authorization.*)

These settings control when authorization checks occur. See TIME_CLOCK.md Part VII for details on timing interactions with authorization.

| Path | Type | Options | Default | Description |
|------|------|---------|---------|-------------|
| `authorization.check_at` | String | "queue", "process" | "process" | When to check authorization |

**check_at options:**
- `"queue"` - check authorization when mutation enters queue (fast rejection, but state may change before processing)
- `"process"` - check authorization when mutation is processed (authorization reflects current state)

## Presets

### database

Traditional database behavior with wall-clock time and strict consistency.

```
time.now_source = "wall"
time.tick_on = "commit"
execution.input_mode = "immediate"
execution.rule_mode = "eager"
execution.quiescence = "strict"
```

Each transaction commit advances logical time by one tick. Rules fire immediately within the transaction and must reach quiescence before commit succeeds.

Use cases: CRUD applications, traditional databases, transactional systems.

### game

Fixed timestep game loop with logical time and batched input.

```
time.now_source = "logical"
time.tick_on = "interval"
time.tick_interval = 16ms
execution.input_mode = "batched"
execution.rule_mode = "eager"
execution.quiescence = "bounded"
```

The `tick_on = "interval"` combined with `tick_interval = 16ms` creates a ~60 Hz game loop. Batched input collects player actions between ticks for deterministic processing.

Use cases: Real-time games, interactive simulations, game servers.

### simulation

Deterministic simulation with manual time control.

```
time.now_source = "logical"
time.tick_on = "manual"
execution.input_mode = "queued"
execution.rule_mode = "eager"
execution.quiescence = "strict"
```

With `tick_on = "manual"`, time only advances via explicit `TICK` statements. Combined with logical time, this enables fully deterministic replay.

Use cases: Deterministic simulations, testing, replay systems.

### streaming

High-throughput event processing with best-effort quiescence.

```
time.now_source = "wall"
time.tick_on = "interval"
time.tick_interval = 100ms
execution.input_mode = "queued"
execution.rule_mode = "deferred"
execution.quiescence = "best_effort"
execution.time_budget = 90ms
```

The 90ms time budget with 100ms tick interval leaves 10ms headroom. Deferred rules and queued input maximize throughput at the cost of latency.

Use cases: Event streaming, log processing, analytics pipelines.

## Layer 0

### Nodes

```
node _TimingConfig [sealed, singleton] {
  now_source: String [in: ["wall", "logical", "hybrid"]] = "wall",
  tick_on: String [in: ["manual", "commit", "interval"]] = "commit",
  tick_interval_ms: Int?,
  doc: String?
}

node _ExecutionConfig [sealed, singleton] {
  input_mode: String [in: ["immediate", "queued", "batched"]] = "immediate",
  rule_mode: String [in: ["eager", "deferred", "explicit"]] = "eager",
  quiescence: String [in: ["strict", "bounded", "best_effort"]] = "strict",
  on_limit: String [in: ["rollback", "commit", "defer"]] = "rollback",
  deterministic: Bool = false,
  max_rule_depth: Int = 100,
  max_rule_actions: Int = 10000,
  max_execution_time_ms: Int = 1000,
  max_queue_size: Int = 10000,
  max_batch_size: Int = 1000,
  time_budget_ms: Int?,
  queue_capacity: Int?,
  batch_window_ms: Int?,
  doc: String?
}

node _TickState [sealed, singleton] {
  global_tick: Int = 0,
  last_tick_at: Timestamp?
}
```

### Edges

```
edge _ontology_has_timing_config(ontology: _Ontology, config: _TimingConfig) {}

edge _ontology_has_execution_config(ontology: _Ontology, config: _ExecutionConfig) {}

edge _ontology_has_tick_state(ontology: _Ontology, state: _TickState) {}
```

### Constraints

```
constraint _singleton_timing_config:
  ontology: _Ontology =>
    COUNT(c: _TimingConfig, _ontology_has_timing_config(ontology, c)) <= 1

constraint _singleton_execution_config:
  ontology: _Ontology =>
    COUNT(c: _ExecutionConfig, _ontology_has_execution_config(ontology, c)) <= 1

constraint _singleton_tick_state:
  ontology: _Ontology =>
    COUNT(s: _TickState, _ontology_has_tick_state(ontology, s)) <= 1
```

## Examples

### Game Server Setup

```
-- Load game preset
CONFIGURE PRESET game

-- Override tick rate for 30 FPS instead of 60
SET time.tick_interval = 33ms

-- Increase rule limits for complex game logic
SET execution.max_rule_actions = 50000
SET execution.max_rule_depth = 200

-- Verify configuration
SHOW CONFIGURATION
```

### Deterministic Testing

```
-- Load simulation preset for deterministic behavior
CONFIGURE PRESET simulation

-- Enable determinism enforcement
SET execution.deterministic = true

-- Now wall_time() usage will error
-- Random functions will error
-- Execution is fully reproducible
```

### High-Throughput Pipeline

```
-- Load streaming preset
CONFIGURE PRESET streaming

-- Tune for specific workload
SET execution.max_queue_size = 100000
SET execution.time_budget = 50ms
SET time.tick_interval = 50ms

-- Check resulting configuration
SHOW CONFIGURATION execution
```

### Inspect and Modify

```
-- Check current timing configuration
SHOW CONFIGURATION time
-- Output:
-- time.now_source = "wall"
-- time.tick_on = "commit"
-- time.tick_interval = null

-- Check specific value
SHOW CONFIGURATION execution.rule_mode
-- Output:
-- execution.rule_mode = "eager"

-- Modify single value
SET execution.rule_mode = "deferred"
```

## Errors

| Code | Condition | Message |
|------|-----------|---------|
| E9011 | Configuration value out of range or wrong type | `INVALID_CONFIG_VALUE: '{path}' expects {expected}, got {actual}` |
| E9012 | Configuration path doesn't exist | `UNKNOWN_CONFIG_PATH: Configuration path '{path}' is not recognized` |
| E9013 | Configuration settings conflict | `INCOMPATIBLE_SETTINGS: '{setting1}' is incompatible with '{setting2}'` |
| E9014 | Preset name not recognized | `UNKNOWN_PRESET: Preset '{name}' is not defined` |
| E9015 | Duration format invalid | `INVALID_DURATION: '{value}' is not a valid duration` |

### Error Examples

```
-- E9011: Wrong type
SET time.now_source = 42
-- Error: INVALID_CONFIG_VALUE: 'time.now_source' expects String, got Int

-- E9011: Invalid option
SET time.now_source = "invalid"
-- Error: INVALID_CONFIG_VALUE: 'time.now_source' expects one of ["wall", "logical", "hybrid"], got "invalid"

-- E9012: Unknown path
SET time.unknown = "value"
-- Error: UNKNOWN_CONFIG_PATH: Configuration path 'time.unknown' is not recognized

-- E9013: Incompatible settings
SET execution.deterministic = true
SET execution.max_execution_time = 1000ms
-- Error: INCOMPATIBLE_SETTINGS: 'execution.deterministic' is incompatible with 'execution.max_execution_time' (use count-based limits)

-- E9014: Unknown preset
CONFIGURE PRESET unknown
-- Error: UNKNOWN_PRESET: Preset 'unknown' is not defined

-- E9015: Invalid duration
SET time.tick_interval = 16
-- Error: INVALID_DURATION: '16' is not a valid duration (missing unit: ms, s, m, h)
```
