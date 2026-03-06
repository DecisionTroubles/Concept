# World Pack Guide

This file explains how to load your own 3D world into the app.

Use this when you want to make your own dataset, not just edit the built-in Japanese one.

## What a world pack is

A world pack is one JSON file that describes:

- what the world is called
- what kinds of nodes exist
- how nodes are grouped
- how nodes connect to each other
- how an opened node is shown page by page

In simple terms:

- `nodes` are the things you see in the 3D world
- `edges` are the lines between them
- `note types` define what a node contains inside
- `layers` help organize visibility and meaning

The app scans known world folders for `pack.json` files, then lets you choose which world to load.

## Where to put it

Create a file like this:

`domains/<your-world>/pack.json`

Example:

`domains/programming/pack.json`

Bundled worlds are scanned from:

`domains/*/pack.json`

User-added worlds can also live in the app data `worlds/` folder.

## Important rules

- the file must be valid JSON
- the file must be saved as UTF-8 **without BOM**
- `version` must be `"2"`
- every id should stay stable once you start using the pack

Why stable ids matter:

- nodes and note types reference each other by id
- changing ids later can break links and migrations

## Small example

```json
{
  "version": "2",
  "world": {
    "id": "programming-core",
    "name": "Programming Core"
  },
  "note_types": [
    {
      "id": "concept-card",
      "name": "Concept Card",
      "schema_json": {
        "version": 1,
        "fields": [
          { "key": "summary", "label": "Summary", "type": "string", "widget": "long_text" },
          { "key": "example", "label": "Example", "type": "string", "widget": "long_text" }
        ]
      },
      "layout_json": {
        "version": 1,
        "pages": [
          {
            "id": "overview",
            "label": "Overview",
            "kind": "content",
            "sections": [
              {
                "id": "main",
                "label": "Main",
                "items": [
                  { "field": "summary" },
                  { "field": "example" }
                ]
              }
            ]
          },
          {
            "id": "connections",
            "label": "Connections",
            "kind": "built_in",
            "source": "connections"
          }
        ]
      }
    }
  ],
  "relation_kinds": [
    {
      "id": "context",
      "label": "Context",
      "directed": false,
      "default_weight": 1.0
    }
  ],
  "layers": [
    {
      "id": "main",
      "name": "Main",
      "display_order": 0
    }
  ],
  "connection_layers": [
    {
      "id": "core-links",
      "name": "Core Links",
      "display_order": 0
    }
  ],
  "nodes": [
    {
      "id": "variables",
      "title": "Variables",
      "node_type": "concept",
      "note_type_id": "concept-card",
      "note_fields": {
        "summary": "Variables store values under a name.",
        "example": "let count = 3"
      },
      "tags": ["basics"],
      "position": { "x": 0, "y": 0, "z": 0 }
    },
    {
      "id": "functions",
      "title": "Functions",
      "node_type": "concept",
      "note_type_id": "concept-card",
      "note_fields": {
        "summary": "Functions group reusable behavior.",
        "example": "function add(a, b) { return a + b }"
      },
      "tags": ["basics"],
      "position": { "x": 4, "y": 0, "z": 0 }
    }
  ],
  "edges": [
    {
      "id": "variables-functions",
      "source_id": "variables",
      "target_id": "functions",
      "relation_id": "context",
      "weight": 1.0
    }
  ]
}
```

## Top-level keys

These are the main sections of a pack:

```json
{
  "version": "2",
  "world": {},
  "note_types": [],
  "relation_kinds": [],
  "layers": [],
  "connection_layers": [],
  "nodes": [],
  "edges": []
}
```

What each one means:

- `world`: basic identity of the world
- `note_types`: templates for what node content looks like
- `relation_kinds`: types of relationships between nodes
- `layers`: main node grouping/visibility system
- `connection_layers`: independent edge overlays
- `nodes`: actual concepts/items in the world
- `edges`: links between nodes

## `world`

Example:

```json
"world": {
  "id": "programming-core",
  "name": "Programming Core",
  "layout": {},
  "metadata": {}
}
```

What it does:

- `id` identifies the world internally
- `name` is what humans see
- `layout` is for world-level layout config
- `metadata` is a place for extra world-level settings

Use `metadata` for things that describe the whole world, not one node.

## `note_types`

This is one of the most important parts.

A note type defines what an opened node contains and how its pages are arranged.

If you are thinking in Anki terms:

- a note type is the reusable content template

Example:

```json
{
  "id": "concept-card",
  "name": "Concept Card",
  "schema_json": {
    "version": 1,
    "fields": [
      { "key": "summary", "label": "Summary", "type": "string", "widget": "long_text" },
      { "key": "example", "label": "Example", "type": "string", "widget": "long_text" }
    ]
  },
  "layout_json": {
    "version": 1,
    "pages": [
      {
        "id": "overview",
        "label": "Overview",
        "kind": "content",
        "sections": [
          {
            "id": "main",
            "label": "Main",
            "items": [
              { "field": "summary" },
              { "field": "example" }
            ]
          }
        ]
      }
    ]
  }
}
```

What each part does:

- `id`: stable internal name
- `name`: human-facing name
- `schema_json`: which fields exist
- `layout_json`: which pages exist and what goes on them

### `schema_json`

This defines the fields a node of that type can store.

Example:

```json
{
  "version": 1,
  "fields": [
    { "key": "summary", "label": "Summary", "type": "string", "widget": "long_text" }
  ]
}
```

What happens if you add a field:

- the node can store that value in `note_fields`
- the authoring UI can edit it
- the node viewer can show it if the layout uses it

### `layout_json`

This defines the pages inside the centered node viewer.

Example:

```json
{
  "version": 1,
  "pages": [
    {
      "id": "overview",
      "label": "Overview",
      "kind": "content",
      "sections": [
        {
          "id": "main",
          "label": "Main",
          "items": [
            { "field": "summary" }
          ]
        }
      ]
    }
  ]
}
```

What happens if you add a page:

- the node viewer gets another `< >` page for that node type

What happens if you move items between pages:

- the node viewer changes order and grouping

### Built-in pages

You can add built-in app pages through layout.

Example:

```json
{
  "id": "learning",
  "label": "Learning",
  "kind": "built_in",
  "source": "learning"
}
```

Supported built-in `source` values:

- `connections`
- `learning`
- `history`

What happens if you add one:

- that built-in page appears in the node viewer as its own page

### Extension pages

You can also place plugin-provided pages into the node viewer.

Example:

```json
{
  "id": "node-notes-page",
  "label": "Notes",
  "kind": "extension",
  "extension_id": "node-notes"
}
```

What happens if you add one:

- the node viewer shows that extension as its own page
- if the extension is missing, that page cannot render properly

Use this for:

- notes
- assets
- AI tools
- domain-specific helpers

## `relation_kinds`

These define what an edge means.

Example:

```json
{
  "id": "context",
  "label": "Context",
  "directed": false,
  "default_weight": 1.0,
  "metadata": {}
}
```

What it does:

- gives a relationship a name and identity
- allows different kinds of connections to exist at the same time
- can carry default style metadata for edges

If you want different relationship meanings like:

- `context`
- `progression`
- `contrast`

this is where you define them.

## `layers`

These are the main node layers.

Example:

```json
{
  "id": "main",
  "name": "Main",
  "display_order": 0
}
```

Think of these as the main grouping system for nodes.

What happens if you add another layer:

- you can place some nodes into that layer
- the UI can treat that as a separate visible node set or grouping

Use node layers when the nodes themselves belong to different groups.

## `connection_layers`

These are separate from node layers.

They describe independent edge overlays.

Example:

```json
{
  "id": "context-links",
  "name": "Context Links",
  "display_order": 0
}
```

What happens if you add another connection layer:

- the user can filter edge overlays separately
- the same pair of nodes can be connected in different ways at the same time

This is useful when you want:

- one overlay for gradual progression
- another overlay for contextual bridges

These are not parent/child layers. They are independent.

## `nodes`

These are the actual things in the world.

Example:

```json
{
  "id": "variables",
  "title": "Variables",
  "node_type": "concept",
  "note_type_id": "concept-card",
  "note_fields": {
    "summary": "Variables store values under a name."
  },
  "tags": ["basics"],
  "position": { "x": 0, "y": 0, "z": 0 },
  "metadata": {}
}
```

What each field means:

- `id`: stable internal id
- `title`: the visible node title
- `node_type`: loose category like `concept`, `word`, `pattern`
- `note_type_id`: which template this node uses
- `note_fields`: the actual structured content
- `tags`: free labels
- `position`: where the node starts in 3D
- `metadata`: extra per-node data

### `note_type_id`

This decides which template controls the opened node.

What happens if you change it:

- the node viewer pages may change
- the editable fields may change
- some old `note_fields` may stop matching the new schema

So do not change note types casually once real content exists.

### `note_fields`

This is the real node content.

Example:

```json
"note_fields": {
  "summary": "Variables store values under a name.",
  "example": "let count = 3"
}
```

What happens if you add a value here:

- the node stores that content
- the viewer can show it if the note type layout includes that field

If the field is not in the note type schema:

- it may be ignored or become hard to work with

Keep `note_fields` aligned with the note type schema.

## `edges`

Edges connect nodes.

Example:

```json
{
  "id": "variables-functions",
  "source_id": "variables",
  "target_id": "functions",
  "relation_id": "context",
  "weight": 1.0,
  "connection_layer_membership": ["context-links"]
}
```

What it does:

- links one node to another
- gives that link a relation kind
- optionally assigns it to one or more connection layers

### `relation_id`

This points to something from `relation_kinds`.

What happens if you change it:

- the meaning of the edge changes
- the style can also change if the relation kind has styling metadata

### `connection_layer_membership`

This lets one edge belong to specific connection overlays.

What happens if you add this:

- that edge becomes part of that edge overlay
- the user can show/hide it through connection layer controls

## Common mistakes

### 1. Saving with BOM

Problem:

- app says `Invalid domain pack JSON: expected value at line 1 column 1`

Cause:

- file saved as UTF-8 with BOM

Fix:

- save as UTF-8 without BOM

### 2. Using a field in layout that does not exist in schema

Problem:

- page looks wrong or empty

Cause:

- `layout_json` references a field not defined in `schema_json`

Fix:

- make schema and layout match

### 3. Changing ids too freely

Problem:

- links or note type references break

Fix:

- treat ids as stable internal keys

### 4. Mixing node layers and connection layers

Problem:

- world structure becomes confusing

Fix:

- use `layers` for node grouping
- use `connection_layers` for edge overlays

## What to do after writing the pack

1. put it in `domains/<your-world>/pack.json`
2. restart the app fully
3. open `Settings -> Worlds`
4. select the world you want to load

If the pack is valid, the app should load your world into the 3D scene.

## Recommended starting point

Use:

- [README.md](C:\Projects\concept\README.md)
- [OVERRIDES.md](C:\Projects\concept\OVERRIDES.md)
- `domains/japanese/pack.json`

Use the Japanese pack as the working example, then replace it with your own world step by step.
