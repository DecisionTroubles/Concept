/// Domain pack loader (v2 only).
///
/// A domain pack is a JSON file that defines a domain-agnostic world with
/// configurable layers, nodes, relations, and edges.
use rusqlite::{params, Connection};
use serde::Deserialize;
use serde_json::Value;

use crate::error::AppError;
use crate::graph::{insert_edge_with_relation, insert_layer, insert_node, CreateNodeInput};

fn now_ts() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

#[derive(Deserialize)]
pub struct DomainPackV2 {
    pub version: String,
    pub world: PackWorldV2,
    #[serde(default)]
    pub relation_kinds: Vec<PackRelationKindV2>,
    pub layers: Vec<PackLayerV2>,
    #[serde(default)]
    pub connection_layers: Vec<PackConnectionLayerV2>,
    pub nodes: Vec<PackNodeV2>,
    #[serde(default)]
    pub edges: Vec<PackEdgeV2>,
}

#[derive(Deserialize)]
pub struct PackWorldV2 {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub layout: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PackRelationKindV2 {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub directed: bool,
    #[serde(default = "default_weight")]
    pub default_weight: f64,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PackLayerV2 {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    #[serde(default)]
    pub node_filter: Value,
    #[serde(default)]
    pub edge_filter: Value,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PackConnectionLayerV2 {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PackNodeV2 {
    pub id: String,
    pub title: String,
    pub node_type: String,
    pub content_data: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_weight")]
    pub weight: f64,
    pub position: Option<PackNodePositionV2>,
    #[serde(default)]
    pub layer_membership: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Deserialize)]
pub struct PackNodePositionV2 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Deserialize)]
pub struct PackEdgeV2 {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_id: String,
    pub edge_type: Option<String>,
    pub weight: Option<f64>,
    #[serde(default)]
    pub connection_layer_membership: Vec<String>,
    #[serde(default)]
    pub metadata: Value,
}

fn default_weight() -> f64 {
    1.0
}

fn json_text(v: &Value) -> String {
    if v.is_null() {
        "{}".to_string()
    } else {
        serde_json::to_string(v).unwrap_or_else(|_| "{}".to_string())
    }
}

fn ensure_world(
    conn: &Connection,
    world_id: &str,
    name: &str,
    config_json: &str,
) -> Result<(), AppError> {
    let created_at = now_ts();
    conn.execute(
        "INSERT OR IGNORE INTO worlds (id, name, config_json, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![world_id, name, config_json, created_at],
    )?;
    Ok(())
}

fn ensure_relation_kind(
    conn: &Connection,
    id: &str,
    world_id: &str,
    label: &str,
    directed: bool,
    default_weight: f64,
    metadata: &str,
) -> Result<(), AppError> {
    let created_at = now_ts();
    conn.execute(
        "INSERT OR IGNORE INTO relation_kinds
             (id, world_id, label, directed, default_weight, metadata, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            world_id,
            label,
            if directed { 1 } else { 0 },
            default_weight,
            metadata,
            created_at
        ],
    )?;
    Ok(())
}

fn seed_v2(conn: &Connection, pack: DomainPackV2) -> Result<(), AppError> {
    if pack.layers.is_empty() || pack.nodes.is_empty() {
        return Err(AppError::Other("Domain pack has no layers or nodes".into()));
    }
    if pack.connection_layers.is_empty() {
        return Err(AppError::Other(
            "Domain pack has no connection_layers".into(),
        ));
    }

    let world_exists: i64 = conn.query_row(
        "SELECT COUNT(*) FROM worlds WHERE id = ?1",
        [&pack.world.id],
        |row| row.get(0),
    )?;
    if world_exists > 0 {
        return Ok(());
    }

    let world_config_json = serde_json::to_string(&serde_json::json!({
        "layout": pack.world.layout.clone(),
        "metadata": pack.world.metadata.clone(),
    }))
    .unwrap_or_else(|_| "{}".to_string());
    ensure_world(conn, &pack.world.id, &pack.world.name, &world_config_json)?;

    for relation in &pack.relation_kinds {
        ensure_relation_kind(
            conn,
            &relation.id,
            &pack.world.id,
            &relation.label,
            relation.directed,
            relation.default_weight,
            &json_text(&relation.metadata),
        )?;
    }

    let relation_label_by_id: std::collections::BTreeMap<String, String> = pack
        .relation_kinds
        .iter()
        .map(|r| (r.id.clone(), r.label.clone()))
        .collect();
    let relation_default_weight: std::collections::BTreeMap<String, f64> = pack
        .relation_kinds
        .iter()
        .map(|r| (r.id.clone(), r.default_weight))
        .collect();

    let mut layer_id_map = std::collections::BTreeMap::<String, String>::new();
    for layer in &pack.layers {
        let inserted = insert_layer(conn, &layer.name, layer.display_order)?;
        let filter = serde_json::json!({
            "node_filter": layer.node_filter.clone(),
            "edge_filter": layer.edge_filter.clone(),
        });
        conn.execute(
            "UPDATE layers SET filter_json = ?1, metadata = ?2 WHERE id = ?3",
            params![json_text(&filter), json_text(&layer.metadata), inserted.id],
        )?;
        layer_id_map.insert(layer.id.clone(), inserted.id);
    }

    let mut connection_layer_id_map = std::collections::BTreeMap::<String, String>::new();
    for connection_layer in &pack.connection_layers {
        let id = uuid::Uuid::new_v4().to_string();
        conn.execute(
            "INSERT INTO connection_layers (id, name, display_order, metadata, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                id,
                connection_layer.name,
                connection_layer.display_order,
                json_text(&connection_layer.metadata),
                now_ts(),
            ],
        )?;
        connection_layer_id_map.insert(connection_layer.id.clone(), id);
    }

    let fallback_layer_id = layer_id_map
        .values()
        .next()
        .ok_or_else(|| AppError::Other("No layers available for node placement".into()))?
        .clone();

    let mut node_id_map = std::collections::BTreeMap::<String, String>::new();
    for node in &pack.nodes {
        let primary_layer_cfg = node.layer_membership.first().cloned();
        let primary_layer = primary_layer_cfg
            .as_ref()
            .and_then(|cfg| layer_id_map.get(cfg))
            .cloned()
            .unwrap_or_else(|| fallback_layer_id.clone());

        let inserted = insert_node(
            conn,
            CreateNodeInput {
                title: node.title.clone(),
                layer_id: primary_layer.clone(),
                node_type: node.node_type.clone(),
                note_type_id: None,
                note_fields: None,
                content_data: node.content_data.clone(),
                tags: node.tags.clone(),
                weight: node.weight,
            },
        )?;

        if let Some(pos) = &node.position {
            conn.execute(
                "UPDATE nodes SET pos_x = ?1, pos_y = ?2, pos_z = ?3 WHERE id = ?4",
                params![pos.x, pos.y, pos.z, inserted.id],
            )?;
        }
        conn.execute(
            "UPDATE nodes SET metadata = ?1 WHERE id = ?2",
            params![json_text(&node.metadata), inserted.id],
        )?;

        if node.layer_membership.is_empty() {
            conn.execute(
                "INSERT OR IGNORE INTO node_layers (node_id, layer_id, created_at) VALUES (?1, ?2, ?3)",
                params![inserted.id, primary_layer, now_ts()],
            )?;
        } else {
            for layer_cfg_id in &node.layer_membership {
                let layer_db_id = layer_id_map.get(layer_cfg_id).ok_or_else(|| {
                    AppError::Other(format!(
                        "Node '{}' references missing layer '{}'",
                        node.title, layer_cfg_id
                    ))
                })?;
                conn.execute(
                    "INSERT OR IGNORE INTO node_layers (node_id, layer_id, created_at) VALUES (?1, ?2, ?3)",
                    params![inserted.id, layer_db_id, now_ts()],
                )?;
            }
        }

        node_id_map.insert(node.id.clone(), inserted.id);
    }

    for edge in &pack.edges {
        if !relation_label_by_id.contains_key(&edge.relation_id) {
            return Err(AppError::Other(format!(
                "Edge '{}' references unknown relation '{}'",
                edge.id, edge.relation_id
            )));
        }

        let source_id = node_id_map.get(&edge.source_id).ok_or_else(|| {
            AppError::Other(format!(
                "Edge '{}' references missing source node '{}'",
                edge.id, edge.source_id
            ))
        })?;
        let target_id = node_id_map.get(&edge.target_id).ok_or_else(|| {
            AppError::Other(format!(
                "Edge '{}' references missing target node '{}'",
                edge.id, edge.target_id
            ))
        })?;

        let edge_type = edge
            .edge_type
            .clone()
            .or_else(|| relation_label_by_id.get(&edge.relation_id).cloned())
            .unwrap_or_else(|| edge.relation_id.clone());

        let inserted = insert_edge_with_relation(
            conn,
            source_id,
            target_id,
            &edge_type,
            Some(&edge.relation_id),
        )?;

        let final_weight = edge
            .weight
            .or_else(|| relation_default_weight.get(&edge.relation_id).copied())
            .unwrap_or(1.0);

        conn.execute(
            "UPDATE edges SET weight = ?1 WHERE id = ?2",
            params![final_weight, inserted.id],
        )?;
        conn.execute(
            "UPDATE edges SET edge_type = ?1 WHERE id = ?2",
            params![edge_type, inserted.id],
        )?;

        if edge.connection_layer_membership.is_empty() {
            let first_connection_layer =
                connection_layer_id_map.values().next().ok_or_else(|| {
                    AppError::Other("No connection layers available for edge membership".into())
                })?;
            conn.execute(
                "INSERT OR IGNORE INTO edge_connection_layers (edge_id, connection_layer_id, created_at)
                 VALUES (?1, ?2, ?3)",
                params![inserted.id, first_connection_layer, now_ts()],
            )?;
        } else {
            for connection_layer_cfg_id in &edge.connection_layer_membership {
                let connection_layer_db_id = connection_layer_id_map
                    .get(connection_layer_cfg_id)
                    .ok_or_else(|| {
                    AppError::Other(format!(
                        "Edge '{}' references missing connection layer '{}'",
                        edge.id, connection_layer_cfg_id
                    ))
                })?;
                conn.execute(
                    "INSERT OR IGNORE INTO edge_connection_layers (edge_id, connection_layer_id, created_at)
                     VALUES (?1, ?2, ?3)",
                    params![inserted.id, connection_layer_db_id, now_ts()],
                )?;
            }
        }

        let _ = &edge.metadata;
    }

    Ok(())
}

/// Parse `json` as a v2 domain pack and write it into `conn`.
///
/// Idempotent by `world.id`.
pub fn seed_pack(conn: &Connection, json: &str) -> Result<(), AppError> {
    let raw: Value = serde_json::from_str(json)
        .map_err(|e| AppError::Other(format!("Invalid domain pack JSON: {e}")))?;

    let version = raw
        .get("version")
        .and_then(Value::as_str)
        .ok_or_else(|| AppError::Other("Domain pack is missing required field 'version'".into()))?;

    if version != "2" {
        return Err(AppError::Other(format!(
            "Unsupported domain pack version '{}'. Only version '2' is supported.",
            version
        )));
    }

    let pack: DomainPackV2 = serde_json::from_value(raw)
        .map_err(|e| AppError::Other(format!("Invalid v2 domain pack: {e}")))?;
    if pack.version != "2" {
        return Err(AppError::Other(format!(
            "Unsupported pack version '{}'. Only version '2' is supported.",
            pack.version
        )));
    }

    seed_v2(conn, pack)
}
