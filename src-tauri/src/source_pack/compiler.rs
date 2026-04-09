use std::collections::{BTreeMap, BTreeSet};

use serde_json::{json, Value};

use crate::domain::{
    DomainPackV2, PackConnectionLayerV2, PackEdgeV2, PackLayerV2, PackNodePositionV2, PackNodeV2, PackNoteTypeV2,
    PackRelationKindV2, PackWorldV2,
};
use crate::error::AppError;

use super::diagnostics::{diagnostic, sort_diagnostics, SourcePackDiagnostics};
use super::markdown::{normalized_heading, parse_markdown_sections};
use super::note_types::{runtime_fields, runtime_layout_json, runtime_schema_json};
use super::toml_value_to_json;
use super::types::{SourceNode, SourceNodeLink, SourcePack, SourceThemeNodeTypeStyle};

const OVERVIEW_KEYS: &[&str] = &["summary", "meaning", "function", "main", "concept"];
const WHY_KEYS: &[&str] = &["why", "whyitmatters", "usage", "when"];
const SIGNAL_KEYS: &[&str] = &["signals", "mapsignals", "tip"];
const PITFALL_KEYS: &[&str] = &["pitfall", "warning", "caution", "note"];
const EXAMPLE_KEYS: &[&str] = &["example", "examplecode", "diagram", "visual"];

fn display_label(id: &str, fallback: Option<&str>) -> String {
    fallback
        .map(str::to_string)
        .unwrap_or_else(|| id.replace('-', " "))
}

fn node_theme_style<'a>(pack: &'a SourcePack, node_type: &str) -> Option<&'a SourceThemeNodeTypeStyle> {
    pack.theme
        .as_ref()
        .and_then(|theme| theme.node_types.get(node_type))
}

fn normalized_field_map(source_node: &SourceNode) -> BTreeMap<String, String> {
    let parsed = parse_markdown_sections(&source_node.body);
    let mut map = BTreeMap::new();
    for section in parsed.sections {
        let body = section.body.trim();
        if body.is_empty() {
            continue;
        }
        map.insert(section.heading, body.to_string());
    }
    map
}

fn direct_or_fallback_fields(
    fields_by_heading: &BTreeMap<String, String>,
    declared_fields: &[String],
    diagnostics: &mut SourcePackDiagnostics,
    file_path: &str,
    node_id: &str,
) -> BTreeMap<String, String> {
    let mut note_fields = BTreeMap::new();
    let mut normalized_to_declared = BTreeMap::new();
    for field in declared_fields {
        normalized_to_declared.insert(normalized_heading(field), field.clone());
    }

    for (heading, body) in fields_by_heading {
        let normalized = normalized_heading(heading);
        let target_key = normalized_to_declared
            .get(&normalized)
            .cloned()
            .or_else(|| {
                let needles = if OVERVIEW_KEYS.iter().any(|needle| normalized.contains(needle)) {
                    OVERVIEW_KEYS
                } else if WHY_KEYS.iter().any(|needle| normalized.contains(needle)) {
                    WHY_KEYS
                } else if SIGNAL_KEYS.iter().any(|needle| normalized.contains(needle)) {
                    SIGNAL_KEYS
                } else if PITFALL_KEYS.iter().any(|needle| normalized.contains(needle)) {
                    PITFALL_KEYS
                } else if EXAMPLE_KEYS.iter().any(|needle| normalized.contains(needle)) {
                    EXAMPLE_KEYS
                } else {
                    &[][..]
                };
                normalized_to_declared
                    .iter()
                    .find(|(field_norm, _)| needles.iter().any(|needle| field_norm.contains(needle)))
                    .map(|(_, declared)| declared.clone())
            });

        if let Some(target_key) = target_key {
            note_fields.insert(target_key.clone(), body.clone());
            if normalized != normalized_heading(&target_key) {
                diagnostics.push(diagnostic(
                    "info",
                    "fallback_field_mapping",
                    format!("Mapped heading '{heading}' to field '{target_key}' via fallback semantics."),
                    Some(file_path.to_string()),
                    None,
                    None,
                    Some(node_id.to_string()),
                ));
            }
        } else {
            diagnostics.push(diagnostic(
                "warning",
                "unrecognized_heading",
                format!("Heading '{heading}' did not match any declared note type field."),
                Some(file_path.to_string()),
                None,
                None,
                Some(node_id.to_string()),
            ));
        }
    }

    note_fields
}

fn connection_layer_display_order(index: usize, explicit: Option<i32>) -> i32 {
    explicit.unwrap_or(index as i32)
}

fn layer_display_order(index: usize, explicit: Option<i32>) -> i32 {
    explicit.unwrap_or(index as i32)
}

fn compile_node_metadata(pack: &SourcePack, node: &SourceNode) -> Value {
    let mut metadata = serde_json::Map::new();

    if let Some(group) = &node.frontmatter.group {
        metadata.insert("group".into(), Value::String(group.clone()));
    }
    if let Some(style_override) = &node.frontmatter.style_override {
        metadata.insert("style_override".into(), toml_value_to_json(style_override));
    }
    if let Some(metadata_value) = &node.frontmatter.metadata {
        metadata.insert("source_metadata".into(), toml_value_to_json(metadata_value));
    }
    if let Some(theme_style) = node_theme_style(pack, &node.frontmatter.node_type) {
        metadata.insert(
            "theme".into(),
            json!({
                "color": theme_style.color,
                "emissive": theme_style.emissive,
                "radius": theme_style.radius,
            }),
        );
    }
    Value::Object(metadata)
}

fn compile_node_position(node: &SourceNode) -> Option<PackNodePositionV2> {
    let placement = node.frontmatter.placement.as_ref()?;
    let (Some(x), Some(y), Some(z)) = (placement.x, placement.y, placement.z) else {
        return None;
    };
    Some(PackNodePositionV2 { x, y, z })
}

fn compile_edge(
    source_id: &str,
    target_id: &str,
    relation_id: &str,
    relation_label: &str,
    link: &SourceNodeLink,
    index: usize,
) -> PackEdgeV2 {
    PackEdgeV2 {
        id: format!(
            "{}__{}__{}__{}",
            source_id,
            relation_id,
            target_id,
            index
        ),
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        relation_id: relation_id.to_string(),
        edge_type: Some(relation_label.to_string()),
        weight: link.weight,
        connection_layer_membership: link.layers.clone(),
        metadata: link
            .metadata
            .as_ref()
            .map(toml_value_to_json)
            .unwrap_or_else(|| json!({})),
    }
}

pub fn compile_source_pack(pack: &SourcePack) -> Result<(DomainPackV2, SourcePackDiagnostics), AppError> {
    let mut diagnostics = SourcePackDiagnostics::new();

    if pack.manifest.version != "source-v1" {
        diagnostics.push(diagnostic(
            "error",
            "unsupported_source_version",
            format!(
                "Unsupported source pack version '{}'. Expected 'source-v1'.",
                pack.manifest.version
            ),
            Some(pack.pack_file.to_string_lossy().to_string()),
            None,
            None,
            None,
        ));
    }

    let mut note_type_ids = BTreeSet::new();
    for note_type in &pack.note_types {
        if !note_type_ids.insert(note_type.id.clone()) {
            diagnostics.push(diagnostic(
                "error",
                "duplicate_note_type_id",
                format!("Duplicate note type id '{}'.", note_type.id),
                None,
                None,
                None,
                Some(note_type.id.clone()),
            ));
        }
    }

    let mut group_ids = BTreeSet::new();
    for group in &pack.groups {
        if !group_ids.insert(group.id.clone()) {
            diagnostics.push(diagnostic(
                "error",
                "duplicate_group_id",
                format!("Duplicate group id '{}'.", group.id),
                None,
                None,
                None,
                Some(group.id.clone()),
            ));
        }
    }

    let mut relation_kind_labels = BTreeMap::new();
    let mut relation_kind_ids = BTreeSet::new();
    let relation_kinds = pack
        .relation_kinds
        .iter()
        .map(|relation| {
            if !relation_kind_ids.insert(relation.id.clone()) {
                diagnostics.push(diagnostic(
                    "error",
                    "duplicate_relation_kind_id",
                    format!("Duplicate relation kind id '{}'.", relation.id),
                    None,
                    None,
                    None,
                    Some(relation.id.clone()),
                ));
            }
            let label = display_label(&relation.id, relation.label.as_deref());
            relation_kind_labels.insert(relation.id.clone(), label.clone());
            PackRelationKindV2 {
                id: relation.id.clone(),
                label,
                directed: relation.directed.unwrap_or(true),
                default_weight: relation.default_weight.unwrap_or(1.0),
                metadata: json!({
                    "description": relation.description,
                    "style": relation.style.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                }),
            }
        })
        .collect::<Vec<_>>();

    let mut layer_ids = BTreeSet::new();
    let layers = if pack.layers.is_empty() {
        vec![PackLayerV2 {
            id: pack
                .manifest
                .authoring
                .default_layer
                .clone()
                .unwrap_or_else(|| "main".into()),
            name: "Main".into(),
            display_order: 0,
            node_filter: json!({}),
            edge_filter: json!({}),
            metadata: json!({
                "layout": pack.manifest.layout.node_spacing,
            }),
        }]
    } else {
        pack.layers
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                if !layer_ids.insert(layer.id.clone()) {
                    diagnostics.push(diagnostic(
                        "error",
                        "duplicate_layer_id",
                        format!("Duplicate layer id '{}'.", layer.id),
                        None,
                        None,
                        None,
                        Some(layer.id.clone()),
                    ));
                }
                PackLayerV2 {
                    id: layer.id.clone(),
                    name: display_label(&layer.id, layer.label.as_deref()),
                    display_order: layer_display_order(index, layer.display_order),
                    node_filter: json!({}),
                    edge_filter: json!({}),
                    metadata: json!({
                        "style": layer.style.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                        "layout": layer.layout.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                    }),
                }
            })
            .collect()
    };

    let mut connection_layer_ids = BTreeSet::new();
    let connection_layers = if pack.connection_layers.is_empty() {
        vec![PackConnectionLayerV2 {
            id: "all-links".into(),
            name: "All links".into(),
            display_order: 0,
            metadata: json!({}),
        }]
    } else {
        pack.connection_layers
            .iter()
            .enumerate()
            .map(|(index, layer)| {
                if !connection_layer_ids.insert(layer.id.clone()) {
                    diagnostics.push(diagnostic(
                        "error",
                        "duplicate_connection_layer_id",
                        format!("Duplicate connection layer id '{}'.", layer.id),
                        None,
                        None,
                        None,
                        Some(layer.id.clone()),
                    ));
                }
                PackConnectionLayerV2 {
                    id: layer.id.clone(),
                    name: display_label(&layer.id, layer.label.as_deref()),
                    display_order: connection_layer_display_order(index, layer.display_order),
                    metadata: json!({
                        "style": layer.style.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                        "layout": layer.layout.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                    }),
                }
            })
            .collect()
    };

    let note_type_by_id = pack
        .note_types
        .iter()
        .map(|note_type| (note_type.id.clone(), note_type))
        .collect::<BTreeMap<_, _>>();

    let note_types = pack
        .note_types
        .iter()
        .map(|note_type| PackNoteTypeV2 {
            id: note_type.id.clone(),
            name: note_type.name.clone(),
            base_note_type_id: note_type.base_note_type_id.clone(),
            fields: runtime_fields(note_type),
            schema_json: runtime_schema_json(note_type),
            layout_json: runtime_layout_json(note_type),
            metadata: note_type.metadata.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
            is_default: note_type.is_default.unwrap_or(false),
        })
        .collect::<Vec<_>>();

    let mut node_ids = BTreeSet::new();
    for node in &pack.nodes {
        if !node_ids.insert(node.frontmatter.id.clone()) {
            diagnostics.push(diagnostic(
                "error",
                "duplicate_node_id",
                format!("Duplicate node id '{}'.", node.frontmatter.id),
                Some(node.file_path.to_string_lossy().to_string()),
                None,
                None,
                Some(node.frontmatter.id.clone()),
            ));
        }
        if !note_type_by_id.contains_key(&node.frontmatter.note_type) {
            diagnostics.push(diagnostic(
                "error",
                "missing_note_type",
                format!(
                    "Node '{}' references missing note type '{}'.",
                    node.frontmatter.id, node.frontmatter.note_type
                ),
                Some(node.file_path.to_string_lossy().to_string()),
                None,
                None,
                Some(node.frontmatter.id.clone()),
            ));
        }
        if let Some(group) = &node.frontmatter.group {
            if !group_ids.contains(group) {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_group",
                    format!("Node '{}' references missing group '{}'.", node.frontmatter.id, group),
                    Some(node.file_path.to_string_lossy().to_string()),
                    None,
                    None,
                    Some(node.frontmatter.id.clone()),
                ));
            }
        }
        if let Some(layer) = &node.frontmatter.layer {
            if !layers.iter().any(|candidate| candidate.id == *layer) {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_layer",
                    format!("Node '{}' references missing layer '{}'.", node.frontmatter.id, layer),
                    Some(node.file_path.to_string_lossy().to_string()),
                    None,
                    None,
                    Some(node.frontmatter.id.clone()),
                ));
            }
        }
    }

    if let Some(root_node) = &pack.manifest.world.root_node {
        if !pack.nodes.iter().any(|node| node.frontmatter.id == *root_node) {
            diagnostics.push(diagnostic(
                "error",
                "missing_root_node",
                format!("pack.toml root_node '{}' does not exist.", root_node),
                Some(pack.pack_file.to_string_lossy().to_string()),
                None,
                None,
                Some(root_node.clone()),
            ));
        }
    }

    let mut nodes = Vec::new();
    let mut edge_keys = BTreeSet::new();
    let mut edges = Vec::new();

    for source_node in &pack.nodes {
        let file_path = source_node.file_path.to_string_lossy().to_string();
        let Some(note_type) = note_type_by_id.get(&source_node.frontmatter.note_type) else {
            continue;
        };
        let fields_by_heading = normalized_field_map(source_node);
        let note_fields = direct_or_fallback_fields(
            &fields_by_heading,
            &runtime_fields(note_type),
            &mut diagnostics,
            &file_path,
            &source_node.frontmatter.id,
        );

        if note_fields.is_empty() {
            diagnostics.push(diagnostic(
                "warning",
                "node_has_no_content_fields",
                format!("Node '{}' has no recognized content sections.", source_node.frontmatter.id),
                Some(file_path.clone()),
                None,
                None,
                Some(source_node.frontmatter.id.clone()),
            ));
        }

        nodes.push(PackNodeV2 {
            id: source_node.frontmatter.id.clone(),
            title: source_node.frontmatter.title.clone(),
            parent_node_id: source_node.frontmatter.parent.clone().filter(|value| !value.trim().is_empty()),
            node_type: source_node.frontmatter.node_type.clone(),
            note_type_id: Some(source_node.frontmatter.note_type.clone()),
            note_fields: serde_json::to_value(&note_fields).unwrap_or_else(|_| json!({})),
            content_data: if source_node.body.trim().is_empty() {
                None
            } else {
                Some(source_node.body.trim().to_string())
            },
            tags: source_node.frontmatter.tags.clone(),
            weight: 1.0,
            position: compile_node_position(source_node),
            layer_membership: vec![source_node
                .frontmatter
                .layer
                .clone()
                .or_else(|| pack.manifest.authoring.default_layer.clone())
                .unwrap_or_else(|| "main".into())],
            sublayer_nodes: Vec::new(),
            metadata: compile_node_metadata(pack, source_node),
        });

        for (link_index, link) in source_node.frontmatter.links.iter().enumerate() {
            if !node_ids.contains(&link.to) {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_link_target",
                    format!(
                        "Node '{}' links to missing node '{}'.",
                        source_node.frontmatter.id, link.to
                    ),
                    Some(file_path.clone()),
                    None,
                    None,
                    Some(source_node.frontmatter.id.clone()),
                ));
                continue;
            }
            if !relation_kind_labels.contains_key(&link.relation) {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_relation_kind",
                    format!(
                        "Node '{}' references missing relation kind '{}'.",
                        source_node.frontmatter.id, link.relation
                    ),
                    Some(file_path.clone()),
                    None,
                    None,
                    Some(source_node.frontmatter.id.clone()),
                ));
                continue;
            }
            if link.layers.is_empty() {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_connection_layers",
                    format!(
                        "Node '{}' has a link to '{}' without connection layers.",
                        source_node.frontmatter.id, link.to
                    ),
                    Some(file_path.clone()),
                    None,
                    None,
                    Some(source_node.frontmatter.id.clone()),
                ));
                continue;
            }
            let mut invalid_layer = false;
            for layer in &link.layers {
                if !connection_layers.iter().any(|candidate| candidate.id == *layer) {
                    diagnostics.push(diagnostic(
                        "error",
                        "missing_connection_layer",
                        format!(
                            "Node '{}' references missing connection layer '{}'.",
                            source_node.frontmatter.id, layer
                        ),
                        Some(file_path.clone()),
                        None,
                        None,
                        Some(source_node.frontmatter.id.clone()),
                    ));
                    invalid_layer = true;
                }
            }
            if invalid_layer {
                continue;
            }

            let edge_key = format!(
                "{}|{}|{}|{}",
                source_node.frontmatter.id,
                link.relation,
                link.to,
                link.layers.join(",")
            );
            if !edge_keys.insert(edge_key.clone()) {
                diagnostics.push(diagnostic(
                    "warning",
                    "duplicate_link",
                    format!(
                        "Duplicate link '{}' on node '{}' was deduplicated.",
                        edge_key, source_node.frontmatter.id
                    ),
                    Some(file_path.clone()),
                    None,
                    None,
                    Some(source_node.frontmatter.id.clone()),
                ));
                continue;
            }

            let relation_label = relation_kind_labels
                .get(&link.relation)
                .cloned()
                .unwrap_or_else(|| link.relation.clone());
            edges.push(compile_edge(
                &source_node.frontmatter.id,
                &link.to,
                &link.relation,
                &relation_label,
                link,
                link_index,
            ));

            if link.bidirectional.unwrap_or(false) {
                let reverse_key = format!(
                    "{}|{}|{}|{}",
                    link.to,
                    link.relation,
                    source_node.frontmatter.id,
                    link.layers.join(",")
                );
                if edge_keys.insert(reverse_key) {
                    edges.push(compile_edge(
                        &link.to,
                        &source_node.frontmatter.id,
                        &link.relation,
                        &relation_label,
                        link,
                        link_index + 10_000,
                    ));
                }
            }
        }
    }

    for node in &nodes {
        if let Some(parent) = &node.parent_node_id {
            if !node_ids.contains(parent) {
                diagnostics.push(diagnostic(
                    "error",
                    "missing_parent",
                    format!("Node '{}' references missing parent '{}'.", node.id, parent),
                    None,
                    None,
                    None,
                    Some(node.id.clone()),
                ));
            }
        }
    }

    for note_type in &pack.note_types {
        let declared = runtime_fields(note_type);
        for field in &declared {
            let populated = nodes.iter().any(|node| {
                node.note_type_id.as_deref() == Some(note_type.id.as_str())
                    && node
                        .note_fields
                        .as_object()
                        .and_then(|fields| fields.get(field))
                        .and_then(Value::as_str)
                        .map(|value| !value.trim().is_empty())
                        .unwrap_or(false)
            });
            if !populated {
                diagnostics.push(diagnostic(
                    "warning",
                    "unused_note_type_field",
                    format!("Note type '{}' declares field '{}' but no node populates it.", note_type.id, field),
                    None,
                    None,
                    None,
                    Some(note_type.id.clone()),
                ));
            }
        }
    }

    sort_diagnostics(&mut diagnostics.diagnostics);

    nodes.sort_by(|a, b| a.id.cmp(&b.id));
    edges.sort_by(|a, b| {
        a.source_id
            .cmp(&b.source_id)
            .then(a.target_id.cmp(&b.target_id))
            .then(a.relation_id.cmp(&b.relation_id))
            .then(a.connection_layer_membership.join(",").cmp(&b.connection_layer_membership.join(",")))
    });

    Ok((
        DomainPackV2 {
            version: "2".into(),
            world: PackWorldV2 {
                id: pack.manifest.world.id.clone(),
                name: pack.manifest.world.name.clone(),
                layout: json!({
                    "mode": pack.manifest.layout.mode,
                    "node_spacing": pack.manifest.layout.node_spacing,
                    "group_spacing": pack.manifest.layout.group_spacing,
                    "focus_child_radius": pack.manifest.layout.focus_child_radius,
                    "allow_explicit_positions": pack.manifest.layout.allow_explicit_positions,
                }),
                metadata: json!({
                    "description": pack.manifest.world.description,
                    "root_node": pack.manifest.world.root_node,
                    "default_note_type": pack.manifest.world.default_note_type,
                    "authoring": {
                        "default_group": pack.manifest.authoring.default_group,
                        "default_layer": pack.manifest.authoring.default_layer,
                    },
                    "theme": pack.theme.as_ref().map(|theme| {
                        json!({
                            "node_types": theme.node_types.iter().map(|(key, style)| (
                                key.clone(),
                                json!({
                                    "color": style.color,
                                    "emissive": style.emissive,
                                    "radius": style.radius,
                                })
                            )).collect::<serde_json::Map<_, _>>(),
                            "labels": theme.labels.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                            "focus": theme.focus.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
                        })
                    }).unwrap_or_else(|| json!({})),
                    "source_pack": {
                        "version": pack.manifest.version,
                    },
                }),
            },
            note_types,
            relation_kinds,
            layers,
            connection_layers,
            nodes,
            edges,
        },
        diagnostics,
    ))
}
