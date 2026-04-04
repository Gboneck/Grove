# Grove OS — Design System

## Philosophy

Grove OS has a warm, dark, premium aesthetic. It should feel like a private journal meets mission control — intimate but powerful. No bright colors. No playful illustrations. No corporate blue.

The visual identity comes from Daemon and must not deviate.

---

## Typography

### Font Stack

| Role | Font | Weight | Usage |
|------|------|--------|-------|
| Display | Instrument Serif | 400 (regular) | Page titles, greeting headers, orb label |
| UI | Syne | 400-700 | Body text, labels, buttons, navigation |
| Data | JetBrains Mono | 400-500 | Metrics, timestamps, code, status values |

### Scale

| Name | Size | Line Height | Font | Usage |
|------|------|-------------|------|-------|
| display-xl | 2.25rem (36px) | 1.2 | Instrument Serif | Main greeting |
| display-lg | 1.5rem (24px) | 1.3 | Instrument Serif | Section headers |
| body-lg | 1.125rem (18px) | 1.6 | Syne | Primary body text |
| body | 1rem (16px) | 1.5 | Syne | Standard body |
| body-sm | 0.875rem (14px) | 1.5 | Syne | Secondary text, labels |
| caption | 0.75rem (12px) | 1.4 | Syne | Timestamps, metadata |
| mono-lg | 1.5rem (24px) | 1.2 | JetBrains Mono | Metric values |
| mono | 0.875rem (14px) | 1.4 | JetBrains Mono | Status, code snippets |
| mono-sm | 0.75rem (12px) | 1.4 | JetBrains Mono | Subtle data |

### Rules
- Never use Inter, Roboto, Arial, or system sans-serif fonts
- Instrument Serif is for display only — never for body text
- JetBrains Mono is for data only — never for prose
- Syne is the workhorse — default for everything else

---

## Color Palette

### Base

| Token | Hex | Usage |
|-------|-----|-------|
| `grove-bg` | `#0a0a0a` | Page background, app shell |
| `grove-surface` | `#141414` | Cards, panels, block backgrounds |
| `grove-surface-hover` | `#1a1a1a` | Hover states on surfaces |
| `grove-border` | `#222222` | Borders, dividers |

### Text

| Token | Hex | Usage |
|-------|-----|-------|
| `grove-text-primary` | `#e5e5e5` | Primary body text |
| `grove-text-secondary` | `#888888` | Secondary text, labels |

### Accent

| Token | Value | Usage |
|-------|-------|-------|
| `grove-accent` | `#d4a853` | Primary gold/amber accent |
| `grove-accent-dim` | `rgba(212, 168, 83, 0.2)` | Subtle accent backgrounds |

### Status

| Token | Hex | Usage |
|-------|-----|-------|
| `grove-status-green` | `#4ade80` | Healthy, on track |
| `grove-status-yellow` | `#facc15` | Warning, needs attention |
| `grove-status-red` | `#f87171` | Critical, blocked |

### Model Indicators

| Token | Hex | Meaning |
|-------|-----|---------|
| `grove-model-local` | `#4ade80` | Running on local Gemma |
| `grove-model-cloud` | `#60a5fa` | Using Claude API |
| `grove-model-offline` | `#6b7280` | No model available |

### Rules
- No pure white (`#ffffff`) — use `#e5e5e5` for text
- No pure black for text — use warm dark backgrounds
- No blue/purple gradients
- Gold/amber is the ONLY accent color
- Status colors used sparingly, only for status indicators

---

## Grain Overlay

A subtle film grain texture overlays the entire app, adding warmth and depth.

```css
.grain-overlay {
  position: fixed;
  inset: 0;
  pointer-events: none;
  z-index: 9999;
  opacity: 0.03;
  background-image: url("data:image/svg+xml,..."); /* noise pattern */
  mix-blend-mode: overlay;
}
```

- Opacity: 0.03 (barely visible, felt more than seen)
- Blend mode: overlay
- Always on top, never interactive
- Subtle enough to not interfere with readability

---

## Daemon Orb

The Daemon Orb is the visual heartbeat of Grove OS. It sits in the header area and communicates system state through animation.

### States

| State | Animation | Color | When |
|-------|-----------|-------|------|
| `idle` | Slow breathe (4s cycle) | Gold dim | App open, no activity |
| `thinking` | Faster pulse (1.5s) | Gold bright | Model processing |
| `listening` | Gentle expand (2s) | Gold warm | Waiting for input |
| `acting` | Quick pulse (0.8s) | Gold + green tint | Executing action |
| `alert` | Sharp pulse (0.5s) | Gold + amber | Needs attention |
| `reflecting` | Ultra slow (6s) | Gold dim + subtle | Heartbeat cycle |
| `offline` | Static, no animation | Gray | No model available |

### Implementation

```
┌─────────────────────┐
│     Outer Glow       │  ← Radial gradient, pulses with state
│  ┌───────────────┐  │
│  │  Inner Circle  │  │  ← Solid gold core, scales with breathe
│  │    (12px)      │  │
│  └───────────────┘  │
│     (24px total)     │
└─────────────────────┘
```

- Outer glow: `box-shadow` with animated spread
- Inner core: `transform: scale()` with CSS animation
- State transitions: 300ms ease-in-out between states
- Position: Header area, left of title or centered

### CSS Animation

```css
@keyframes orb-breathe {
  0%, 100% { transform: scale(1); opacity: 0.7; }
  50% { transform: scale(1.15); opacity: 1; }
}

@keyframes orb-think {
  0%, 100% { transform: scale(1); opacity: 0.8; }
  50% { transform: scale(1.25); opacity: 1; }
}

@keyframes orb-act {
  0%, 100% { transform: scale(1); }
  25% { transform: scale(1.3); }
  50% { transform: scale(0.95); }
  75% { transform: scale(1.15); }
}
```

---

## Block Styling

### Card Pattern

All blocks share a base card style:

```
Background: grove-surface (#141414)
Border: 1px solid grove-border (#222222)
Border radius: 8px (rounded-lg)
Padding: 16px (p-4)
Margin bottom: 12px (mb-3)
```

### Block-Specific Accents

| Block | Left Border | Icon/Accent |
|-------|-------------|-------------|
| Text (heading) | None | Instrument Serif font |
| Text (body) | None | Syne font |
| Metric | Gold left border (2px) | JetBrains Mono value |
| Actions | None | Gold text on action items |
| Status | None | Colored dots (green/yellow/red) |
| Insight | Gold left border (2px) | Subtle gold background |
| Input | Gold focus ring | Syne placeholder |
| Progress | None | Gold progress bar fill |
| Quote | Gold left border (3px) | Italic Instrument Serif |
| List | None | Gold bullets/numbers |

### Streaming Animation

New blocks enter with:
```css
/* Initial state */
transform: translateY(8px);
opacity: 0;

/* Animate to */
transform: translateY(0);
opacity: 1;
transition: all 500ms ease-out;
```

---

## Spacing System

Using Tailwind's default 4px grid:

| Token | Value | Usage |
|-------|-------|-------|
| `space-1` | 4px | Tight inline spacing |
| `space-2` | 8px | Between related elements |
| `space-3` | 12px | Between blocks |
| `space-4` | 16px | Card padding, section gaps |
| `space-6` | 24px | Major section breaks |
| `space-8` | 32px | Page-level spacing |

---

## Interaction Patterns

### Hover
- Surface elements: `grove-surface` → `grove-surface-hover` (200ms)
- Text elements: opacity 0.7 → 1.0 (200ms)
- Accent elements: brightness increase (200ms)

### Focus
- Input fields: `ring-1 ring-grove-accent` (gold focus ring)
- Buttons: `ring-1 ring-grove-accent/50` (subtle gold)

### Active/Press
- Scale: `transform: scale(0.98)` (100ms)
- Opacity: slight dim

### Modal/Panel
- Backdrop: `bg-black/60 backdrop-blur-md`
- Panel: slides in or fades in (300ms)
- Close: Escape key, backdrop click, or X button

---

## Ambient Themes

The model can set an ambient mood that subtly shifts the color temperature:

| Mood | Shift | Usage |
|------|-------|-------|
| `warm` | Slightly warmer tones | Morning, reflection |
| `cool` | Slightly cooler tones | Focus, deep work |
| `neutral` | Default palette | Standard state |
| `alert` | Subtle amber warmth | Deadlines approaching |

These are CSS variable overrides, not theme switches. The shift is subtle — felt, not noticed.
