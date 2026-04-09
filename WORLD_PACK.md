# World Pack Guide

This guide explains the current pack workflow in plain terms.

The short version:

- author packs as a source-pack folder
- keep long content in Markdown
- keep metadata and structure in TOML
- let the app/compiler produce runtime `pack.json`

Do not start by hand-writing one giant `pack.json` unless you are only maintaining legacy data.

## The Mental Model

A world pack has four kinds of information:

1. graph structure
2. node content
3. visual/layout hints
4. compiled runtime data

The source-pack format splits those up so the files stay readable.

## Source Pack Folder

Use this structure:

```txt
your-world/
  pack.toml
  theme.toml
  note-types/
    *.toml
  relation-kinds/
    *.toml
  layers/
    *.toml
  connection-layers/
    *.toml
  groups/
    *.toml
  nodes/
    *.md
```

Rules:

- `pack.toml` is required
- `nodes/*.md` are required
- config-like files use TOML
- node content uses Markdown with TOML frontmatter
- runtime `pack.json` is generated from this source pack

## Why This Format

It is easier for both humans and LLMs because:

- one node lives in one file
- real content is readable Markdown
- metadata is structured but not noisy
- diffs are small
- links are local to the node that owns them

## What Goes Where

### `pack.toml`

World identity and defaults.

Example:

```toml
version = "source-v1"

[world]
id = "starter-example"
name = "Starter Example"
description = "Small starter world."
root_node = "welcome"
default_note_type = "starter-concept"

[authoring]
default_group = "core"
default_layer = "main"

[layout]
mode = "force"
node_spacing = 7.0
group_spacing = 18.0
focus_child_radius = 8.0
allow_explicit_positions = true

[build]
emit_runtime_pack = true
runtime_pack_version = "2"
```

Important:

- `version` must be `source-v1`
- `root_node` must point to a real node
- runtime pack version stays `2`

## `theme.toml`

World-level visual defaults.

Example:

```toml
[node_types.concept]
color = "#8fb3ff"
emissive = "#314e93"
radius = 1.0
```

Use this for:

- node colors
- emissive defaults
- shared visual identity

## `groups/*.toml`

Visual/layout clustering for nodes.

Example:

```toml
id = "types"
label = "Types"

[style]
color = "#38bdf8"
emissive = "#155e75"

[layout]
cohesion = 1.2
intra_spacing = 7.0
inter_spacing = 18.0
```

Use groups for:

- local visual identity
- layout cohesion hints

Do not use groups as a replacement for edges.

## `layers/*.toml`

World navigation layers.

Example:

```toml
id = "main"
label = "Main"
```

Use layers for:

- world-level visibility organization
- layer metadata

## `connection-layers/*.toml`

How links are styled and grouped visually.

Example:

```toml
id = "gradual"
label = "Gradual"

[style]
color = "#f59e0b"
width = 2.0
opacity = 0.85
shape = "arc"
dash_size = 0.0
gap_size = 0.0
```

Use connection layers for:

- edge color
- width
- line style
- edge grouping

## `relation-kinds/*.toml`

Semantic meaning of a relation.

Example:

```toml
id = "progression"
label = "Progression"
description = "A recommended next concept."
directed = true
default_weight = 1.0
```

Keep this separate from connection layers:

- relation kind = meaning
- connection layer = how it is shown

## `note-types/*.toml`

Node schema and viewer page structure.

Example:

```toml
id = "concept-basic"
name = "Concept Basic"
is_default = true

[[fields]]
key = "Summary"
label = "Summary"
type = "string"
widget = "text"

[[fields]]
key = "Details"
label = "Details"
type = "string"
widget = "markdown"

[[fields]]
key = "Example"
label = "Example"
type = "string"
widget = "long_text"

[[pages]]
id = "overview"
label = "Overview"
kind = "content"
fields = ["Summary", "Details"]

[[pages]]
id = "example"
label = "Example"
kind = "content"
fields = ["Example"]

[[pages]]
id = "connections"
label = "Connections"
kind = "built_in"
source = "connections"
```

This compiles into the runtime note type structure automatically.

## `nodes/*.md`

This is where most real authoring happens.

Each node is:

- TOML frontmatter
- Markdown body

Example:

```md
+++
id = "enum"
title = "enum"
node_type = "concept"
note_type = "concept-basic"
group = "types"
layer = "main"
tags = ["odin", "state"]
parent = ""

[placement]
locked = false

[[links]]
to = "declaration"
relation = "progression"
layers = ["gradual"]
+++

# Summary
Finite named set of values.

# Details
Enums make state explicit and safe.

# Example
```odin
Color :: enum {
    Red,
    Green,
    Blue,
}
```
```

### Required frontmatter fields

- `id`
- `title`
- `node_type`
- `note_type`

### Common optional fields

- `group`
- `layer`
- `tags`
- `parent`
- `[placement]`
- `[[links]]`
- `[style_override]`
- `[metadata]`

## Links

Links are authored per node.

Example:

```toml
[[links]]
to = "package"
relation = "semantic"
layers = ["concept"]

[[links]]
to = "enum-state-vocabulary"
relation = "sublayer"
layers = ["focus"]
```

Each link becomes a runtime edge.

Use:

- `to` for the target node id
- `relation` for meaning
- `layers` for edge display grouping

Optional:

- `weight`
- `bidirectional`
- `metadata`

## Placement

Source packs are hints-first, not coordinates-first.

That means:

- most nodes should not need explicit coordinates
- spacing usually belongs to world/group/layer hints
- explicit coordinates are only for exceptions

Example:

```toml
[placement]
x = 4.0
y = 0.0
z = -2.0
locked = true
```

Use explicit placement sparingly.

## Markdown Headings -> Note Fields

The compiler maps headings into note fields.

Preferred direct mappings:

- `# Overview`
- `# Summary`
- `# Why`
- `# Example`
- `# Pitfall`
- `# Signals`

If the note type does not have an exact field match, the compiler applies fallback semantic mapping.

So:

- `Summary`, `Meaning`, `Function`, `Main`, `Concept` tend to map into overview-like fields
- `Why`, `WhyItMatters`, `Usage`, `When` tend to map into explanation fields
- `Example`, `ExampleCode`, `Diagram`, `Visual` tend to map into example fields

Unknown headings are warned about instead of silently disappearing.

## The Actual Runtime

Runtime still loads canonical `pack.json` v2.

That is intentional.

The workflow is:

1. author source pack
2. validate source pack
3. compile source pack
4. load runtime `pack.json`

So `pack.json` is still important, but it is a build artifact, not the preferred authoring format.

## Local World Creation In The App

When you create a new local world in the app now, it creates:

- a source-pack folder
- a compiled `pack.json`

Important:

- “blank” world means minimal valid scaffold, not zero files
- it still includes one root node so the pack stays valid

## Legacy Runtime Packs

Old raw `pack.json` packs still work.

Use them when:

- you already have one
- you are maintaining compatibility

Do not choose them as the default format for new work.

## Recommended Workflow

For a new world:

1. create a new local world in the app, or create a source-pack folder yourself
2. edit `pack.toml`
3. define one note type
4. define one layer and one connection layer
5. author nodes in `nodes/*.md`
6. validate/compile
7. load the compiled world

Start small:

- one note type
- one group
- one layer
- one relation kind
- 3 to 10 nodes

Then expand.

## What Not To Do

Avoid:

- one giant hand-written `pack.json`
- putting all content into TOML strings
- repeating colors/layout numbers on every node
- inventing a custom DSL
- using explicit coordinates everywhere

## Current Limitation

In-app node editing still edits the runtime world state.

That means:

- source-pack authoring is now the preferred disk format
- but in-app editing does not yet round-trip back into source-pack files

That is a later phase.
