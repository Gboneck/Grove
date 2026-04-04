# Grove OS — 5-Session Roadmap

Each session is ~8 prompts, ending with a PR.

---

## Session 1: Architecture Assessment + Foundation
**Branch**: `claude/grove-os-assessment-eOkCw`
**PR**: "feat: architecture assessment + soul parser + visual identity"

### Deliverables
1. CLAUDE.md — Project constitution
2. docs/ASSESSMENT.md — Full codebase audit
3. docs/ARCHITECTURE.md — Refined architecture spec
4. docs/DESIGN.md — Design system specification
5. docs/ROADMAP.md — This document
6. Soul.md structured parser (Rust) — Section extraction with confidence scores
7. Daemon Orb component — 7-state breathing animation
8. Grain overlay — Subtle film grain CSS effect
9. YAML role system — Scaffold with 4 role configs
10. Heartbeat module stubs — Directory structure ready for Session 2
11. Missing directory scaffolding — tests/, roles/, grove-data/, styles/

### State After Session 1
- Full documentation suite exists
- Soul.md is parsed into structured sections (not raw text)
- Visual identity complete (Orb + grain)
- Role configs defined (not yet wired to reasoning)
- Codebase ready for heartbeat implementation

---

## Session 2: Heartbeat + Memory Architecture
**PR**: "feat: always-on heartbeat + structured memory"

### Deliverables
1. Background heartbeat loop (Rust tokio timer)
2. Observer module — file watcher (notify crate), time context, system state
3. Scheduler — tick interval management, observation queue
4. Three-tier memory — ephemeral (session), working (MEMORY.md), long-term (patterns/)
5. MEMORY.md journal — append-on-significant-event, read at cycle start
6. Memory decay — working memory promotes or decays based on relevance
7. Pattern detection foundation — detect recurring time/behavior patterns
8. useHeartbeat hook — subscribe to heartbeat events in React
9. useAmbient hook — theme/mood state management
10. Tests for heartbeat and memory modules

### State After Session 2
- Grove runs a background loop even when idle
- Memory has structure (not flat JSON)
- Patterns begin accumulating
- MEMORY.md serves as cross-session context

---

## Session 3: Autonomy + Intelligence Layer
**PR**: "feat: autonomy scoring + soul evolution + MCP intelligence"

### Deliverables
1. 5-factor autonomy scoring gate (reversibility, scope, confidence, precedent, urgency)
2. Soul evolution — 9-phase relationship arc (awakening → mastery)
3. Progressive disclosure — early interactions observe, later interactions act
4. Soul.md patcher — apply confidence-scored updates after significant observations
5. Sub-agent spawning — Gemma draft → Claude refine for strategic decisions
6. Enhanced MCP tools — "what's priority?", "what changed?", "what should I focus on?"
7. YAML role wiring — roles modify system prompt and block preferences
8. Role switching UI — model or user can change modes
9. Tests for autonomy scoring and soul evolution

### State After Session 3
- Actions go through a scoring gate before execution
- Soul.md evolves with usage (confidence scores change)
- System relationship deepens over time
- MCP is a queryable intelligence layer
- Roles modify reasoning behavior

---

## Session 4: Self-Evolution + Deployment
**PR**: "feat: self-evolution engine + docker deployment + security"

### Deliverables
1. Self-evolution engine — propose prompt/config changes → judge model validates → apply
2. docker-compose.yaml — app + ollama + qdrant containers
3. Headless/API-only mode — Grove as a background service
4. Security hardening:
   - Input validation on all command parameters
   - Shell command sanitization in plugin registry and action executor
   - Path traversal prevention (canonicalize, deny symlinks outside sandbox)
   - Rate limiting on reasoning calls
5. Full test suite — unit tests for all Rust modules
6. Error boundary component (React)
7. Plugin data sources — implement HTTP and shell data source types

### State After Session 4
- Grove can run as Docker service (headless)
- Security gaps closed
- Self-evolution refines its own prompts
- Full test coverage
- Plugin system fully functional

---

## Session 5: Polish + Ship
**PR**: "feat: production polish + README + v1.0"

### Deliverables
1. README.md — Product-quality with screenshots, demo outputs, Soul.md examples
2. Extract Modal component — DRY up 7 panel implementations
3. Accessibility pass — aria-labels, focus traps, keyboard nav audit
4. Performance — React.memo on blocks, cache Soul.md reads, memoize context building
5. Replace 3 `any` types with proper interfaces
6. Move inline hex colors to Tailwind tokens
7. Log rotation (prune logs older than 30 days)
8. End-to-end integration tests
9. Finalize all docs (ARCHITECTURE.md, DESIGN.md verified against implementation)
10. Version bump to 1.0

### State After Session 5
- Production-ready application
- Documentation matches implementation
- README is a first-class deliverable
- Clean, accessible, performant codebase
- Ready for public repo

---

## Success Criteria

After all 5 sessions, Grove OS should:

1. **Run locally** with Gemma 4 as default model, Claude as escalation
2. **Know the user** via Soul.md that evolves with every interaction
3. **Remember** across sessions via structured three-tier memory
4. **Observe** via always-on heartbeat that detects patterns
5. **Compose** unique UI every reasoning cycle based on context
6. **Gate** autonomous actions through confidence scoring
7. **Expose** itself as MCP server for other tools to query
8. **Deploy** via Tauri (desktop) or Docker (server)
9. **Self-improve** via evolution engine that refines its own prompts
10. **Look beautiful** with Daemon visual identity (Orb, grain, warm dark palette)
