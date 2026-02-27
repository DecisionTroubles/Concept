/// Domain pack loader.
///
/// A "domain pack" is a JSON file that describes a self-contained knowledge
/// domain (layers + nodes + edges). The format is intentionally simple so any
/// domain (languages, sciences, history, …) can be expressed without touching
/// Rust code.
///
/// The `seed_pack` function is idempotent: it checks whether the pack has
/// already been loaded by looking for a layer whose name matches the first
/// layer in the pack definition.
use rusqlite::Connection;
use serde::Deserialize;

use crate::error::AppError;
use crate::graph::{insert_edge, insert_layer, insert_node, CreateNodeInput};

// ---------------------------------------------------------------------------
// Pack schema
// ---------------------------------------------------------------------------

#[derive(Deserialize)]
pub struct PackMeta {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize)]
pub struct PackLayer {
    pub name: String,
    pub display_order: i32,
}

#[derive(Deserialize)]
pub struct PackNode {
    pub title: String,
    /// Index into the `layers` array.
    pub layer: usize,
    pub node_type: String,
    pub content_data: Option<String>,
    pub tags: Vec<String>,
    pub weight: f64,
}

#[derive(Deserialize)]
pub struct PackEdge {
    /// Index into the `nodes` array (source).
    pub from: usize,
    /// Index into the `nodes` array (target).
    pub to: usize,
    /// Edge type string: "Context" | "Prerequisite" | "Semantic" | "UserDefined"
    #[serde(rename = "type")]
    pub edge_type: String,
}

#[derive(Deserialize)]
pub struct DomainPack {
    pub meta: PackMeta,
    pub layers: Vec<PackLayer>,
    pub nodes: Vec<PackNode>,
    pub edges: Vec<PackEdge>,
}

// ---------------------------------------------------------------------------
// Loader
// ---------------------------------------------------------------------------

/// Parse `json` as a [`DomainPack`] and write its data into `conn`.
///
/// Idempotent: skips silently if the first layer name is already present.
pub fn seed_pack(conn: &Connection, json: &str) -> Result<(), AppError> {
    let pack: DomainPack = serde_json::from_str(json)
        .map_err(|e| AppError::Other(format!("Invalid domain pack: {e}")))?;

    if pack.layers.is_empty() || pack.nodes.is_empty() {
        return Err(AppError::Other("Domain pack has no layers or nodes".into()));
    }

    // Idempotency: skip if the first layer name already exists.
    let first_name = &pack.layers[0].name;
    let already_seeded: i64 = conn.query_row(
        "SELECT COUNT(*) FROM layers WHERE name = ?1",
        [first_name],
        |row| row.get(0),
    )?;
    if already_seeded > 0 {
        return Ok(());
    }

    // -- Layers ---------------------------------------------------------------
    let mut layer_ids: Vec<String> = Vec::with_capacity(pack.layers.len());
    for layer_def in &pack.layers {
        let layer = insert_layer(conn, &layer_def.name, layer_def.display_order)?;
        layer_ids.push(layer.id);
    }

    // -- Nodes ----------------------------------------------------------------
    let mut node_ids: Vec<String> = Vec::with_capacity(pack.nodes.len());
    for node_def in &pack.nodes {
        let layer_id = layer_ids
            .get(node_def.layer)
            .ok_or_else(|| {
                AppError::Other(format!(
                    "Node '{}' references layer index {} which does not exist",
                    node_def.title, node_def.layer
                ))
            })?
            .clone();

        let node = insert_node(
            conn,
            CreateNodeInput {
                title: node_def.title.clone(),
                layer_id,
                node_type: node_def.node_type.clone(),
                content_data: node_def.content_data.clone(),
                tags: node_def.tags.clone(),
                weight: node_def.weight,
            },
        )?;
        node_ids.push(node.id);
    }

    // -- Edges ----------------------------------------------------------------
    for edge_def in &pack.edges {
        let source_id = node_ids.get(edge_def.from).ok_or_else(|| {
            AppError::Other(format!("Edge 'from' index {} out of range", edge_def.from))
        })?;
        let target_id = node_ids.get(edge_def.to).ok_or_else(|| {
            AppError::Other(format!("Edge 'to' index {} out of range", edge_def.to))
        })?;
        insert_edge(conn, source_id, target_id, &edge_def.edge_type)?;
    }

    Ok(())
}
