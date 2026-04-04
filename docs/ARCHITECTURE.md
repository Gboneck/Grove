# Grove OS — Architecture

## System Overview

Grove OS is a local-first personal operating system. The only hardcoded layer is plumbing — everything the user sees is decided by a reasoning model based on Soul.md and current context.

## Core Principles

1. **Living Code** — No predefined screens. The model composes UI from block primitives every cycle.
2. **Sovereignty First** — All data local. Default model runs locally (Gemma 4 via Ollama). Cloud is opt-in escalation.
3. **Always-On** — Background heartbeat observes even when the app isn't focused.
4. **Progressive Trust** — Capabilities unlock as Soul.md confidence increases.
5. **Intent Over Tasks** — The system holds intents ("stabilize revenue") and derives tasks, not the reverse.

## File Structure

```
grove-os/
├── CLAUDE.md                    # Project constitution
├── soul.md                      # User identity document
├── context.json                 # Venture/project state
├── src-tauri/                   # Rust backend (Tauri)
│   ├── src/
│   │   ├── main.rs              # Entry point
│   │   ├── lib.rs               # Plugin + command registration
│   │   ├── models/              # Dual-model reasoning layer
│   │   │   ├── mod.rs           # Shared types
│   │   │   ├── router.rs        # Confidence-based routing
│   │   │   ├── gemma.rs         # Ollama/local model client
│   │   │   ├── claude.rs        # Anthropic API client
│   │   │   ├── context.rs       # Context builder (Soul + state → prompt)
│   │   │   ├── config.rs        # Model configuration
│   │   │   └── streaming.rs     # Incremental JSON block parser
│   │   ├── commands/            # Tauri command handlers
│   │   │   ├── reason.rs        # Core reasoning cycle
│   │   │   ├── soul.rs          # Soul.md CRUD + structured parsing
│   │   │   ├── identity.rs      # Identity generation
│   │   │   ├── context.rs       # context.json CRUD
│   │   │   ├── memory.rs        # Multi-layer memory
│   │   │   ├── ventures.rs      # Venture state management
│   │   │   ├── autonomous.rs    # Auto-action execution
│   │   │   ├── reflection.rs    # Weekly digest
│   │   │   ├── profiles.rs      # Multi-profile
│   │   │   ├── setup.rs         # Onboarding flow
│   │   │   ├── watch.rs         # File watcher
│   │   │   ├── actions.rs       # Action executor
│   │   │   ├── mcp.rs           # MCP integration (9 tools)
│   │   │   ├── roles.rs         # YAML role loader + switching (Session 3)
│   │   │   ├── logs.rs          # Reasoning logs
│   │   │   └── system.rs        # System info
│   │   ├── heartbeat/           # Always-on background loop (Session 2)
│   │   │   ├── mod.rs
│   │   │   ├── observer.rs      # File watcher, time, system state
│   │   │   ├── scheduler.rs     # Tick interval + queue
│   │   │   └── patterns.rs      # Ambient pattern detection
│   │   ├── soul/                # Soul.md management (Sessions 1-4)
│   │   │   ├── mod.rs
│   │   │   ├── parser.rs        # Structured section parsing
│   │   │   ├── patcher.rs       # Confidence-scored updates
│   │   │   ├── evolution.rs     # 9-phase relationship tracking
│   │   │   ├── autopatch.rs     # Keyword-based insight extraction
│   │   │   └── evolve.rs        # Self-evolution engine (propose/judge/apply)
│   │   ├── memory/              # Three-tier memory (Session 2)
│   │   │   ├── mod.rs
│   │   │   ├── ephemeral.rs     # Current session
│   │   │   ├── working.rs       # Recent days (MEMORY.md)
│   │   │   └── longterm.rs      # Persistent patterns
│   │   ├── autonomy/            # Decision gates (Session 3)
│   │   │   ├── mod.rs           # gate_actions() with 5-factor scoring
│   │   │   └── scoring.rs       # AutonomyScore composite
│   │   ├── security.rs          # Input/path/command/URL validation (Session 4)
│   │   └── plugins/             # Plugin system
│   │       ├── mod.rs
│   │       ├── loader.rs
│   │       ├── registry.rs
│   │       └── manifest.rs
│   └── tauri.conf.json
├── src/                         # React frontend
│   ├── App.tsx                  # Root — no router
│   ├── main.tsx                 # DOM mount
│   ├── index.css                # Fonts + Tailwind + globals
│   ├── lib/tauri.ts             # IPC wrapper
│   ├── components/
│   │   ├── GroveShell.tsx       # Outer chrome + input
│   │   ├── BlockRenderer.tsx    # Block type router
│   │   ├── ModelIndicator.tsx   # Model status dot
│   │   ├── DaemonOrb.tsx        # Breathing pulse (7 states)
│   │   ├── CommandPalette.tsx   # Cmd+K
│   │   ├── SoulEditor.tsx       # Soul.md editor
│   │   ├── NavMenu.tsx          # Navigation
│   │   ├── LoadingState.tsx     # Thinking state
│   │   ├── ActionLog.tsx        # Toast notifications
│   │   ├── SetupScreen.tsx      # Onboarding
│   │   ├── IdentityWizard.tsx   # Soul generation
│   │   ├── RoleSwitcher.tsx     # Role switching dropdown (Session 3)
│   │   ├── Modal.tsx            # Shared modal wrapper (Session 5)
│   │   ├── ErrorBoundary.tsx    # React error boundary (Session 5)
│   │   ├── blocks/              # 10 block types
│   │   └── panels/              # 7 panel types (using Modal)
│   ├── hooks/                   # Custom hooks (Session 2+)
│   │   ├── useReasoning.ts
│   │   ├── useHeartbeat.ts
│   │   └── useAmbient.ts
│   └── styles/                  # Dedicated style files
│       ├── grain.css            # Grain overlay
│       └── animations.css       # Orb + block animations
├── roles/                       # YAML reasoning modes
│   ├── builder.yaml
│   ├── reflector.yaml
│   ├── planner.yaml
│   └── coach.yaml
├── grove-data/                  # User data (~/.grove/)
│   ├── soul.md
│   ├── memory.md
│   ├── config.toml
│   └── memory/
│       ├── observations/
│       ├── decisions/
│       └── patterns/
├── docs/
│   ├── ASSESSMENT.md
│   ├── ARCHITECTURE.md
│   ├── DESIGN.md
│   └── ROADMAP.md
└── tests/
    ├── models/
    ├── soul/
    ├── memory/
    └── heartbeat/
```

## Data Flow

### Reasoning Cycle (Core Loop)

```
Input (user text / timer / file change)
    │
    ▼
Context Builder
    ├── Reads soul.md (structured sections)
    ├── Reads context.json (ventures)
    ├── Reads memory (recent sessions, facts, patterns)
    ├── Reads current role (YAML config)
    └── Assembles system prompt + user message
    │
    ▼
Model Router
    ├── Classifies intent (fast/deep/strategic)
    ├── Checks model availability (Ollama running?)
    ├── Routes to Gemma (local) or Claude (cloud)
    └── Fallback chain: preferred → alternate → error
    │
    ▼
Streaming Parser
    ├── Receives token stream from model
    ├── Extracts JSON blocks incrementally
    └── Emits blocks via Tauri events as they complete
    │
    ▼
Side Effects
    ├── memory.record(session, facts, patterns)
    ├── autonomous.execute(actions with scoring gate)
    ├── ventures.update(if model proposes changes)
    └── soul.patch(if confidence threshold met)
    │
    ▼
BlockRenderer (Frontend)
    ├── Receives blocks via event listener
    ├── Routes each block to typed component
    ├── Applies streaming fade-in animation
    └── Renders final composed UI
```

### Heartbeat Cycle (Background, Session 2)

```
Timer (every 5 minutes, configurable)
    │
    ▼
Observer
    ├── Check file modifications in watched dirs
    ├── Check time-of-day context (morning/afternoon/evening)
    ├── Check system state (battery, network, active app)
    └── Check venture deadlines approaching
    │
    ▼
Pattern Detector
    ├── Compare observations against history
    ├── Detect recurring patterns ("works on X every morning")
    └── Generate whispers (quiet observations)
    │
    ▼
Queue
    ├── Append observations to heartbeat queue
    ├── If queue threshold met → trigger reasoning cycle
    └── Else → wait for next tick or user interaction
```

## Dual-Model Router

### Routing Rules

| Condition | Route To | Reason |
|-----------|----------|--------|
| Offline / Ollama unavailable | Gemma only (or error) | Sovereignty first |
| UI composition, greetings, memory lookups | Gemma (local) | Fast, free, private |
| Journaling, reflection, file reactions | Gemma (local) | Personal, iterative |
| Multi-venture planning (3+ projects) | Claude (cloud) | Needs broad context |
| Code generation, debugging | Claude (cloud) | Superior capability |
| Complex synthesis, prioritization | Claude (cloud) | Deep reasoning needed |
| Gemma confidence < 0.7 | Escalate to Claude | Quality gate |
| User says "think hard" / "go deep" | Claude (cloud) | Explicit escalation |
| Strategic decisions (rare) | Both: Gemma drafts, Claude refines | Dual-pass mode |

### Gemma 4 Model Selection

| System RAM | Model | Notes |
|-----------|-------|-------|
| 32GB+ | gemma4:31b | Dense, max intelligence |
| 16-32GB | gemma4:26b-moe | Best ratio, recommended default |
| 8-16GB | gemma4:e4b | Solid, fast |
| <8GB | gemma4:e2b | Basic reasoning |

## Block System

The model outputs a JSON array of blocks. Each block has a `type` and type-specific fields.

| Block Type | Purpose | Fields |
|-----------|---------|--------|
| `text` | Prose, greetings, observations | `content`, `style?` (heading/body/subtle) |
| `metric` | Single metric with trend | `label`, `value`, `trend?` (up/down/flat) |
| `actions` | Clickable action list | `items[]` with `label`, `action`, `params?` |
| `status` | Multi-item status row | `items[]` with `name`, `status` (green/yellow/red) |
| `insight` | Observation or pattern | `content`, `confidence?`, `source?` |
| `input` | User text input | `placeholder`, `action` |
| `divider` | Visual separator | (none) |
| `progress` | Progress bar | `label`, `value` (0-100), `color?` |
| `list` | Bullet/number list | `items[]`, `ordered?` |
| `quote` | Attributed quote | `content`, `source?` |

## Autonomy Scoring

Every autonomous action goes through a 5-factor gate before execution:

| Factor | Weight | Examples |
|--------|--------|---------|
| **Reversibility** | High | Can this be undone? File write > email send |
| **Scope** | High | Local file vs external API vs financial |
| **Confidence** | Medium | Model's self-assessed certainty |
| **Precedent** | Medium | Has the user approved similar actions before? |
| **Urgency** | Low | Time-sensitive vs can-wait |

| Action Category | Default Gate |
|----------------|-------------|
| UI composition | Auto — always allowed |
| Memory updates | Auto — always allowed |
| File operations in ~/.grove/ | Auto — within sandbox |
| File operations elsewhere | Ask — requires confirmation |
| External API calls | Ask — requires confirmation |
| Shell commands | Ask — requires confirmation |
| Purchases / sends / messages | Block — never auto |
| System changes | Ask — requires confirmation |

## Memory Architecture

### Three Tiers

| Tier | Scope | Storage | Lifecycle |
|------|-------|---------|-----------|
| **Ephemeral** | Current session | React state + Rust session struct | Cleared on app close |
| **Working** | Recent days (7-30) | MEMORY.md + ~/.grove/memory/ | Decays, significant events promoted |
| **Long-term** | Persistent patterns | ~/.grove/memory/patterns/ (future: vector DB) | Permanent until contradicted |

### Memory Types

| Type | Description | Example |
|------|------------|---------|
| **Episodic** | Session records | "2026-04-04: Worked on Grove architecture for 2 hours" |
| **Semantic** | Facts with confidence | "Grif prefers copy-paste outputs" (0.95) |
| **Procedural** | Behavioral patterns | "Usually works on EMBER Mon/Wed mornings" |

## Soul.md Structure

```markdown
# Soul.md — {Name}

## Identity [confidence: 0.9]
Core identity paragraph.

## Active Ventures [confidence: 0.85]
- **Venture Name** — Description. Status. Next action.

## Current State [confidence: 0.8]
- Time-sensitive observations
- Resource constraints
- Active applications/deadlines

## Work Style [confidence: 0.75]
- Preferences
- Risk patterns
- Energy patterns

## Priority Stack [confidence: 0.9]
1. Top priority — rationale
2. Second priority — rationale

## Relationships [confidence: 0.5]
- Key people and their roles

## Patterns [confidence: 0.6]
- Observed behavioral patterns (grows over time)

## Aspirations [confidence: 0.4]
- Long-term goals and directions
```

Each section has a confidence score (0.0-1.0) that increases with observations and explicit confirmations, and decays with time and contradictions.

## Role System

Roles are YAML configs that modify the reasoning cycle's system prompt and behavior.

```yaml
# roles/reflector.yaml
name: reflector
display: "Reflector"
description: "Journaling, synthesis, and introspection mode"
system_prompt_prefix: |
  You are in reflector mode. Focus on synthesis, patterns, and meaning.
  Ask questions that help the user think deeper. Avoid action items unless asked.
block_preferences:
  - text (heading + body)
  - quote
  - insight
  - input (journaling prompts)
avoid_blocks:
  - metric
  - progress
  - actions
autonomy_level: low  # Don't take actions in reflection mode
```
