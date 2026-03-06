# UI

## Purpose

The UI is designed around one primary idea:

- the 3D world is the main surface

Everything else exists to support navigation, reading, learning, and authoring without replacing the graph as the center of the experience.

## UI Principles

- `World-first`: the graph stays visible and meaningful, not just a launcher for forms
- `Focused overlays`: major tasks open in clear, bounded windows
- `Content over inspection`: opened nodes should read like content pages, not settings panels
- `Consistent shells`: overlays should share framing, spacing, and motion language
- `Theme-defined visuals`: colors, surfaces, borders, text, and emphasis should come from theme variables, not ad hoc component styling
- `Override-friendly`: themes and UI modules must remain replaceable through the plugin system

## Primary Surfaces

### 1. 3D World

The world is the persistent base layer.

It is responsible for:

- node placement and navigation
- connection visibility
- context through spatial grouping
- immediate recognition of selected, pinned, and progress-marked nodes

The world should never feel visually secondary to the overlays placed on top of it.

### 2. Side Node Summary

The side summary is a compact contextual panel for the selected node.

It exists for:

- title and note type
- quick status and learning cues
- immediate actions
- short connection context

It is intentionally not the deep-reading experience.

### 3. Centered Node Viewer

The centered node viewer is the focused reading surface for a node.

It should behave like:

- a buffer-like content window
- page or slide navigation
- one page visible at a time

It should not behave like:

- a settings inspector
- a dense property sheet
- a tab-heavy admin panel

### 4. Buffers

Buffers are alternate workspaces.

Examples:

- pinned nodes
- map buffer
- future review queues or cluster views

Buffers are not the same concept as the centered node viewer. They are broader context surfaces and should remain visually distinct from node reading.

### 5. Node Editor

The node editor is a dedicated authoring workspace for the focused node.

It is separate from both:

- the side summary
- the centered node viewer

Its purpose is:

- edit node fields
- switch note type
- adjust fallback content and tags
- preview how the node renders

This keeps node authoring out of Settings and avoids turning the reader/viewer into a form surface.

### 6. Supporting Overlays

Supporting overlays include:

- settings
- learning/progress window
- search
- mode indicators
- HUD elements

These should stay secondary to both the world and the node viewer.

## Node Viewer Model

The node viewer is page-based.

Page order should come from note type layout and authored page definitions.

Possible page sources:

- note type content pages
- built-in pages such as `connections`, `learning`, `history`
- extension pages such as notes, assets, or future AI tools

Important rule:

- different tools should remain distinct pages when they represent distinct tasks

Examples:

- `Notes` should be its own page
- `Assets` should be its own page
- `AI Assistant` should be its own page

They should not be collapsed into one generic extension catch-all unless a layout explicitly chooses that.

## Theme System

Theme defines the UI globally.

That includes:

- background and overlay surfaces
- text colors
- borders
- focus states
- accent colors
- interactive control appearance
- component contrast and emphasis
- graph-adjacent overlay styling

Components should derive their visuals from theme variables instead of hardcoded colors.

### Theme Scope

A theme is expected to cover:

- app canvas background
- overlay backgrounds
- overlay borders
- primary and secondary text
- accent color
- muted and disabled states
- inputs
- selects
- textareas
- range sliders
- buttons
- chips / badges / markers

This is important because incomplete theme coverage creates mismatched UI, especially on native controls.

## User Theme Overrides

Themes are not core-only.

Users should be able to:

- add new themes
- override existing theme values
- ship themes through user plugins

That means the UI design must assume:

- theme variables are the contract
- components must consume those variables consistently
- new user themes should not require component rewrites

Theme additions and overrides belong in the plugin/override system, not in scattered component-local hacks.

## Plugin and Override UI Contract

The UI is designed to be extensible in two ways:

### Module overrides

Plugins may replace major UI modules such as:

- settings panel
- buffer overlay
- node viewer module
- search surface

### UI extensions

Plugins may add:

- node viewer pages
- custom panels
- themes

This means UI design must preserve:

- stable surface boundaries
- stable slot names
- stable theme variable contracts

Without those, user overrides become brittle.

## Visual Language

### World

- immersive and legible
- dark or low-clutter base
- node emphasis through color, glow, labels, and motion restraint
- edge styling should communicate relation meaning without dominating the scene

### Overlays

- clear separation from the world
- readable typography
- strong information hierarchy
- bounded, intentional surfaces rather than generic white-box forms

### Node Viewer

- centered, calm, content-forward
- large readable stage
- page navigation should feel obvious
- tools and extensions should feel like dedicated spaces, not appended clutter

## Interaction Rules

- selecting a node should stay lightweight
- opening a node should transition into focused reading
- buffer interactions should not be confused with node reading
- keyboard navigation should respect the active surface
- theming should remain coherent across base controls and custom components

## Authoring Surface

Global authoring can live in Settings, but focused node editing is a separate task surface.

It should not leak into the node viewer in a way that turns the viewer back into an editor/settings form.

The design direction for authoring is:

- schema-driven editing
- stronger preview behavior
- page composition awareness
- eventual richer widgets for structured fields and media

## UI Boundaries

To keep the system coherent:

- world navigation is not authoring
- node viewing is not settings
- buffers are not node pages
- themes are global styling contracts
- plugin UI must enter through explicit extension points
