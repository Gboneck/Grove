# Grove OS — CLAUDE.md

## What This Is
Grove OS is a local-first personal operating system where the only hardcoded layer is plumbing.
Everything the user sees is decided by a reasoning model based on Soul.md and current context.

## Architecture Principles
- **Living code**: No predefined screens or features. The model composes UI from primitives.
- **Sovereignty first**: User data stays local. The model runs locally when possible (Gemma 4 via Ollama).
- **Always-on**: Background heartbeat observes and queues, not just when the user opens the app.
- **Progressive trust**: Capabilities unlock as the system learns more about the user.

## Code Style
- Rust backend: Follow standard Rust conventions. Use `thiserror` for errors. Async with `tokio`.
- React frontend: Functional components with hooks. Tailwind for styling. No component libraries.
- TypeScript: Strict mode. No `any`. Named exports.
- All files < 300 lines. Split when approaching this limit.

## Fonts (non-negotiable)
- Display: Instrument Serif
- UI: Syne
- Data/code: JetBrains Mono

## Colors (non-negotiable)
- Base: Warm dark (#0a0a0a backgrounds, #1a1a1a surfaces)
- Primary accent: Gold/amber (#d4a853)
- Secondary: Warm gray tones
- Model indicator: Green (#4ade80) local, Blue (#60a5fa) cloud, Gray (#6b7280) offline

## Testing
- Write tests for every module in src-tauri/src/
- Frontend: Test block rendering with sample model outputs
- Use `cargo test` for Rust, vitest for React

## Git Workflow
- One branch per logical feature
- Conventional commits (feat:, fix:, refactor:, docs:)
- PR after each coherent chunk of work (5-8 prompts per session)

## What NOT To Do
- Do not add a sidebar, nav bar, router, or multiple pages
- Do not use shadcn, MUI, or any component library
- Do not hardcode any content — everything comes from the reasoning model
- Do not use Inter, Roboto, Arial, or system fonts anywhere
- Do not use purple/blue gradient backgrounds
- Do not add authentication in the MVP
- Do not import react-router

## File Structure
See docs/ARCHITECTURE.md for the canonical file tree.

## Key Files
- `soul.md` — User identity document. Read by every reasoning cycle.
- `context.json` — Venture/project state. Drives reasoning priorities.
- `src-tauri/src/models/router.rs` — Dual-model routing (Gemma local + Claude cloud).
- `src/components/BlockRenderer.tsx` — Maps model JSON output to React components.
- `src/components/GroveShell.tsx` — Outer chrome, input bar, theme layer.

## Dual-Model Routing Rules
1. OFFLINE → Gemma 4 always
2. FAST PATH (Gemma 4): UI composition, soul lookups, memory retrieval, journaling
3. ESCALATE TO CLAUDE: Multi-venture planning, code gen, complex synthesis, confidence < 0.7
4. DUAL PASS (rare): Strategic decisions — Gemma drafts, Claude refines

## Memory Architecture
- **Ephemeral**: Current session state (React state + Rust session)
- **Working**: Recent days (memory.md, ~/.grove/memory/)
- **Long-term**: Persistent patterns (future: vector DB)

## Autonomy Scoring (5-factor gate)
- UI composition → auto
- File operations → ask
- External API calls → ask
- Purchases/sends → block
- System changes → ask
