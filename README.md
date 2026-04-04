# Grove OS

A local-first personal operating system where the only hardcoded layer is plumbing. Everything the user sees is decided by a reasoning model based on Soul.md and current context.

## Philosophy

- **No predefined screens.** The model composes UI from typed blocks every reasoning cycle.
- **Sovereignty first.** User data stays local. The model runs locally when possible.
- **Always-on.** A background heartbeat observes and queues, not just when the app is open.
- **Progressive trust.** Capabilities unlock as the system learns more about the user.

## Architecture

```
┌─────────────────────────────────────────┐
│              React Frontend              │
│  GroveShell → BlockRenderer → Blocks    │
│  RoleSwitcher · DaemonOrb · Panels      │
├─────────────────────────────────────────┤
│           Tauri IPC Bridge               │
├─────────────────────────────────────────┤
│              Rust Backend                │
│  Router ─┬─ Gemma 4 (Ollama, local)    │
│          └─ Claude API (cloud)          │
│  Soul Parser · Evolution Engine          │
│  Autonomy Gate · Heartbeat · Memory     │
│  Security · MCP Server                   │
└─────────────────────────────────────────┘
```

### Dual-Model Routing

| Condition | Model |
|-----------|-------|
| Offline | Gemma 4 always |
| Fast path (UI, lookups, journals) | Gemma 4 |
| Deep reasoning (planning, code gen) | Claude |
| Confidence < 0.7 | Escalate to Claude |
| Strategic decisions | Dual pass: Gemma drafts, Claude refines |

### Memory Architecture

| Tier | Storage | Lifetime |
|------|---------|----------|
| Ephemeral | React state + Rust session | Current session |
| Working | MEMORY.md journal | Recent days |
| Long-term | JSON entries (~/.grove/memory/longterm/) | Persistent |

### Self-Evolution Engine

After every reasoning cycle, the system:
1. **Proposes** changes to Soul.md from model insights, confirmed patterns, and confidence decay
2. **Judges** proposals against phase-gated safety rules
3. **Applies** approved patches with conservative confidence deltas

### 9-Phase Relationship Arc

Awakening → Discovery → Deepening → Challenge → Synthesis → Integration → Evolution → Mastery → Transcendence

Each phase adjusts system prompt, autonomy level (0.0 → 0.95), and allowed capabilities.

## Setup

### Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [Node.js](https://nodejs.org/) (18+)
- [Ollama](https://ollama.ai/) (for local model)
- Tauri 2 CLI: `cargo install tauri-cli --version "^2"`

### Install

```bash
git clone https://github.com/Gboneck/Grove.git
cd Grove
npm install
```

### Configure

```bash
# Pull the local model
ollama pull gemma3:4b

# (Optional) Set Claude API key for cloud reasoning
mkdir -p ~/.grove
echo "ANTHROPIC_API_KEY=sk-ant-..." > ~/.grove/.env
```

### Run

```bash
# Development
cargo tauri dev

# Production build
cargo tauri build
```

### Docker (Headless)

```bash
# Start Grove MCP + Ollama + Qdrant
docker compose up -d

# Or run just the MCP server
cargo run --bin grove-mcp              # stdio mode (for Claude Code)
cargo run --bin grove-mcp -- --http 8377  # HTTP API mode
```

### MCP Integration

Add to Claude Code's MCP config:

```json
{
  "mcpServers": {
    "grove": {
      "command": "grove-mcp",
      "args": []
    }
  }
}
```

Available tools: `grove_get_context`, `grove_get_soul`, `grove_get_memory`, `grove_get_facts`, `grove_add_fact`, `grove_get_ventures`, `grove_get_priority`, `grove_what_changed`, `grove_get_focus`

## Key Files

| File | Purpose |
|------|---------|
| `soul.md` | User identity with confidence-scored sections |
| `context.json` | Venture/project state driving reasoning |
| `MEMORY.md` | Cross-session working memory journal |
| `roles/*.yaml` | Reasoning role configs (builder, reflector, planner, coach) |
| `src-tauri/src/models/router.rs` | Dual-model routing with escalation |
| `src-tauri/src/soul/evolve.rs` | Self-evolution engine (propose/judge/apply) |
| `src-tauri/src/autonomy/mod.rs` | 5-factor autonomy scoring gate |
| `src-tauri/src/security.rs` | Input validation, path/command/URL sanitization |
| `src/components/BlockRenderer.tsx` | Maps model JSON to React blocks |
| `src/components/GroveShell.tsx` | App chrome, input bar, theme |

## Visual Identity

- **Display font:** Instrument Serif
- **UI font:** Syne
- **Code font:** JetBrains Mono
- **Background:** Warm dark (#0a0a0a / #1a1a1a)
- **Accent:** Gold/amber (#d4a853)
- **Model indicators:** Green (local), Blue (cloud), Gray (offline)

## Tech Stack

- **Desktop:** Tauri 2 (Rust + WebView)
- **Frontend:** React, TypeScript, Tailwind CSS
- **Backend:** Rust, Tokio, Serde
- **Local LLM:** Ollama (Gemma 4)
- **Cloud LLM:** Anthropic Claude API
- **Memory:** File-based (JSON, Markdown), future: Qdrant vector DB

## License

Private. All rights reserved.
