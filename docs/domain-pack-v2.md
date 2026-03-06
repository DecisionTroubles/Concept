# Domain Pack v2 Specification

This document defines the only supported domain pack format.

## Goals

- Domain-agnostic (not tied to language learning)
- Config-first world definition
- Flexible layering for nodes and connections
- Stable IDs for safe evolution and diffing

## Top-level shape

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

## Required fields

- `version`: must be exactly `"2"`
- `world.id`: unique world identifier (used for idempotent seeding)
- `world.name`: display name
- `layers`: at least 1 layer
- `connection_layers`: at least 1 layer
- `nodes`: at least 1 node

## World

```json
"world": {
  "id": "my-world",
  "name": "My Knowledge Map",
  "layout": {},
  "metadata": {}
}
```

- `layout` and `metadata` are arbitrary JSON objects for config extension.
- Recommended: place global visual defaults in `world.metadata.visual_defaults`.

## Relation kinds

```json
{
  "id": "context",
  "label": "Context",
  "directed": false,
  "default_weight": 1.0,
  "metadata": {}
}
```

- `id` is referenced by `edges[].relation_id`.
- Optional relation style can be declared at `metadata.style` (e.g. edge color/width).
- Typical edge style keys: `color`, `width`, `opacity`, `dash_size`, `gap_size`, `animated_flow`, `flow_speed`.

## Node Layers

```json
{
  "id": "core",
  "name": "Core",
  "display_order": 0,
  "node_filter": {},
  "edge_filter": {},
  "metadata": {}
}
```

- Layer IDs are referenced by `layer_membership` on nodes.
- Optional node style can be declared at `metadata.node_style`.
- Typical node style keys: `color`, `emissive`, `emissive_intensity`.

## Connection Layers

```json
{
  "id": "core-links",
  "name": "Core Links",
  "display_order": 0,
  "metadata": {}
}
```

- Connection layer IDs are referenced by `connection_layer_membership` on edges.
- Optional edge style can be declared at `metadata.edge_style`.
- When an edge belongs to multiple active connection layers, highest `display_order` style wins.

## Nodes

```json
{
  "id": "n-1",
  "title": "Topic A",
  "node_type": "concept",
  "note_type_id": "pack-note-type",
  "note_fields": { "Front": "Topic A", "Back": "Meaning" },
  "content_data": "optional text",
  "tags": ["tag"],
  "weight": 1.0,
  "position": { "x": 0, "y": 0, "z": 0 },
  "connection_layer_membership": ["core-links"],
  "metadata": {}
}
```

- `note_type_id` may reference a pack-defined note type or a stable global base note type.
- `note_fields` is the primary structured node payload for page-driven viewers.
- `layer_membership` is optional, but recommended.
- If omitted, backend assigns the node to a fallback layer.

## Note Types

```json
{
  "id": "pack-note-type",
  "name": "Concept Card",
  "base_note_type_id": "basic",
  "fields": ["Front", "Back", "Example"],
  "schema_json": {
    "version": 1,
    "fields": [
      { "key": "Front", "label": "Front", "type": "string", "widget": "text" },
      { "key": "Back", "label": "Back", "type": "string", "widget": "long_text" }
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
            "items": [{ "field": "Front" }, { "field": "Back" }]
          }
        ]
      },
      { "id": "connections", "label": "Connections", "kind": "built_in", "source": "connections" },
      { "id": "notes", "label": "Node Notes", "kind": "extension", "extension_id": "node-notes" }
    ]
  },
  "metadata": {},
  "is_default": false
}
```

- `base_note_type_id` supports global base templates plus pack-local overrides.
- `schema_json` defines fields and widgets.
- `layout_json` defines page order for the node viewer.
- `kind: "built_in"` currently supports `connections`, `learning`, and `history`.
- `kind: "extension"` renders a specific node extension page by `extension_id`.

## Edges

```json
{
  "id": "e-1",
  "source_id": "n-1",
  "target_id": "n-2",
  "relation_id": "context",
  "edge_type": "Context",
  "weight": 1.0,
  "layer_membership": ["core"],
  "metadata": {}
}
```

- `relation_id` must exist in `relation_kinds`.
- `edge_type` is optional compatibility label used by current UI.

## Validation behavior

Import fails fast when:

- `version` is missing or not `"2"`
- `layers` or `nodes` are empty
- an edge references a missing node
- an edge references an unknown `relation_id`
- a node references a missing layer in `layer_membership`
- an edge references a missing connection layer in `connection_layer_membership`

## Notes

- v1 packs are no longer supported.
- Use `domains/japanese/pack.json` as the current reference example.
