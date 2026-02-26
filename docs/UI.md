# UI Specification — 3D Memory Graph Platform

## Philosophy

The UI is a **dark, immersive environment** — not a traditional app with sidebars and navbars. The 3D graph world is the primary surface. Everything else (descriptions, notes, menus) floats over it as contextual overlays that appear when needed and disappear when not.

The user should feel like they are **navigating a space**, not using software.

---

## UI Framework

### Choice: shadcn-vue (built on Radix Vue)

**Recommended over PrimeVue** for the following reasons:

| Concern | shadcn-vue | PrimeVue |
|---------|------------|----------|
| Dark-first | Yes — dark is the default, not a theme | Dark themes exist but feel secondary |
| Tailwind integration | Native (components are Tailwind classes) | Parallel system, can conflict |
| Bundle size | Zero — you copy only what you use | Full library import |
| Canvas coexistence | No global styles that bleed into Three.js | Has global CSS resets that can interfere |
| Customizability | Full source ownership, modify anything | Override system, more friction |
| Aesthetic | Modern, minimal, premium | Heavier, more "enterprise" |

shadcn-vue components are copied into the project as source (not an npm package), making them fully owned and customizable. Components are built on **Radix Vue** primitives for accessibility.

Install reference: `https://www.shadcn-vue.com/`

---

## Layout Structure

```
┌─────────────────────────────────────────────────────┐
│                                                     │
│          3D Graph Canvas (Three.js)                 │
│          — full viewport, always behind —           │
│                                                     │
│   ┌─────────────┐              ┌─────────────────┐  │
│   │ Layer Panel │              │  Node Detail    │  │
│   │ (floating,  │              │  Panel          │  │
│   │  left edge) │              │  (floating,     │  │
│   └─────────────┘              │  right, on      │  │
│                                │  node click)    │  │
│                                └─────────────────┘  │
│   ┌──────────────────────────────────────────────┐  │
│   │  Status bar / breadcrumb (bottom, minimal)   │  │
│   └──────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────┘
```

### Panels
All panels are:
- **Floating** — `position: fixed`, with glassmorphism background (`backdrop-blur` + semi-transparent dark)
- **Non-blocking** — clicking through the canvas when no panel is focused
- **Dismissible** — pressing Escape or clicking away closes them
- **Minimal by default** — expand on hover/click

### Color palette (dark theme)

```
Background (canvas bg):  #080b14  — near-black with blue undertone
Panel bg:                rgba(12, 16, 28, 0.75) + backdrop-blur-md
Panel border:            rgba(255, 255, 255, 0.06)
Text primary:            #e8eaf0
Text secondary:          #7a8099
Accent (interactive):    #5b8fff  — cool blue
Learned node:            #3dd68c  — emerald green
Edge default:            rgba(90, 100, 140, 0.4)
Edge highlighted:        rgba(91, 143, 255, 0.9)
Edge learned:            rgba(61, 214, 140, 0.6)
```

---

## 3D Node Visual Language

Nodes are not all the same shape. Their geometry encodes their **type/context**:

| Node Type | Geometry | Rationale |
|-----------|----------|-----------|
| Vocabulary / word | Sphere | Smooth, neutral, most common |
| Grammar point | Octahedron | Angular, structural — grammar is the skeleton |
| Kanji / character | Box (slightly rounded) | Dense, contained — kanji are building blocks |
| Concept / abstract | Icosahedron | Complex, multi-faced |
| Connector / particle | Torus (small) | A ring — particles link things |
| Root / anchor node | Larger sphere with ring | Prominent, foundational |

### Node States (color + material)

| State | Visual |
|-------|--------|
| Unseen | Dark grey, low emissive, ~0.3 opacity |
| Seen / in-progress | Mid-tone layer color, full opacity |
| **Learned** | Emerald green (`#3dd68c`), stronger emissive glow |
| **Reachable next** | Accent blue (`#5b8fff`) pulse animation on connected edges |
| Selected (clicked) | White-outlined, brightest emissive, info panel opens |
| Hover | Slight scale-up + brightness increase |

### Node size
- Scaled by `weight` field: more important/common nodes are slightly larger
- Min size prevents disappearing; max size prevents dominating the scene

---

## Edge Visual Language

| Edge Type | Style |
|-----------|-------|
| Context | Solid thin line, default color |
| Prerequisite | Dashed or dotted line |
| Semantic | Thin, slightly transparent |
| UserDefined | Distinct color (amber) |

### Edge state
- **Default**: dim, barely visible (`rgba(90, 100, 140, 0.4)`)
- **Node selected**: edges from that node brighten and show direction arrows
- **Reachable next**: edges to unlearned connected nodes pulse blue
- **Fully learned path**: edges between learned nodes turn green-tinted

---

## Floating Panels

### Node Detail Panel (right side, on click)
```
┌───────────────────────────────┐
│ [Node title]          [×]     │
│ ─────────────────────────── │
│ [Content: text / audio / img] │
│                               │
│ Tags: [grammar] [N5]          │
│                               │
│ Connections:                  │
│   → Node A  (Context)         │
│   → Node B  (Prerequisite)    │
│                               │
│ [Mark as Learned]             │
└───────────────────────────────┘
```
- Opens on node click
- `Mark as Learned` button changes node color immediately (optimistic update to Rust backend)
- After marking learned, connected unlearned edges pulse to guide next steps

### Layer Panel (left side, collapsible)
```
┌────────────┐
│ Layers     │
│ ● Grammar  │  ← active
│ ○ Kanji    │
│ ○ Vocab    │
│            │
│ [+ New]    │
└────────────┘
```
- Clicking a layer switches the active graph
- Cross-layer links are shown as faint connectors

### Minimap (bottom-right, optional)
- 2D top-down projection of the 3D graph
- Shows user's current camera focus position
- Learned nodes shown in green

---

## Camera & Navigation

- **Orbit controls**: drag to rotate, scroll to zoom, right-drag to pan
- **Click node**: camera smoothly animates (lerp) to focus on that node, detail panel opens
- **Double-click empty space**: deselect, camera relaxes back
- **Layer transition**: animated fly-through effect when switching layers
- No hard clip planes — the world feels infinite

---

## Post-Processing (@tresjs/post-processing)

| Effect | Purpose |
|--------|---------|
| Bloom | Glow around emissive nodes (learned, selected, highlighted) |
| Depth of field (subtle) | Focus on selected node, blur distant ones slightly |
| Vignette | Darken edges of viewport for immersive feel |
| FXAA | Anti-aliasing |

---

## Typography

- **Font**: `Inter` (or system-ui fallback) — clean, readable at small sizes
- Panel titles: `text-sm font-semibold tracking-wide uppercase text-secondary`
- Node labels (3D): rendered as sprites (HTML canvas → texture) or CSS2DObject — white, small, visible from ~medium distance only

---

## Interaction Summary

| Action | Result |
|--------|--------|
| Click node | Open detail panel, focus camera |
| Mark as Learned | Node turns green, edges to next nodes pulse |
| Hover node | Scale up, show label |
| Click layer | Switch active graph layer |
| Scroll | Zoom in/out |
| Drag | Rotate view |
| Escape | Close panel, deselect |
| Right-drag | Pan camera |
