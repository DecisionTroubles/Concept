use rusqlite::{params, Connection};
use std::collections::BTreeMap;
use uuid::Uuid;

use crate::error::AppError;

// ---------------------------------------------------------------------------
// Timestamp helper — stored as Unix epoch seconds (TEXT column)
// ---------------------------------------------------------------------------
fn now_ts() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

// ---------------------------------------------------------------------------
// IPC types (Serialize + Deserialize + specta::Type via taurpc macro)
// ---------------------------------------------------------------------------

#[taurpc::ipc_type]
pub struct Layer {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    pub filter_json: String,
    pub metadata: String,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct WorldConfig {
    pub id: String,
    pub name: String,
    pub config_json: String,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct RelationKind {
    pub id: String,
    pub world_id: String,
    pub label: String,
    pub directed: bool,
    pub default_weight: f64,
    pub metadata: String,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct ConnectionLayer {
    pub id: String,
    pub name: String,
    pub display_order: i32,
    pub metadata: String,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct EdgeRef {
    pub id: String,
    pub target_id: String,
    pub edge_type: String,
    pub relation_id: Option<String>,
    pub connection_layer_ids: Vec<String>,
    pub weight: f64,
}

#[taurpc::ipc_type]
pub struct Node {
    pub id: String,
    pub title: String,
    pub layer_id: String,
    pub node_type: String,
    pub note_type_id: Option<String>,
    pub note_fields: BTreeMap<String, String>,
    pub content_type: String,
    pub content_data: Option<String>,
    pub tags: Vec<String>,
    pub learned: bool,
    pub weight: f64,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
    pub pos_z: Option<f64>,
    /// Outgoing edges — loaded in the same call, no second IPC round-trip needed.
    pub connections: Vec<EdgeRef>,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct CreateNodeInput {
    pub title: String,
    pub layer_id: String,
    pub node_type: String,
    pub note_type_id: Option<String>,
    pub note_fields: Option<BTreeMap<String, String>>,
    pub content_data: Option<String>,
    pub tags: Vec<String>,
    pub weight: f64,
}

#[taurpc::ipc_type]
pub struct NoteType {
    pub id: String,
    pub name: String,
    pub fields: Vec<String>,
    pub is_default: bool,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct Edge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub relation_id: Option<String>,
    pub weight: f64,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Layer operations
// ---------------------------------------------------------------------------

pub fn query_layers(conn: &Connection) -> Result<Vec<Layer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, display_order, filter_json, metadata, created_at FROM layers ORDER BY display_order",
    )?;
    let layers = stmt
        .query_map([], |row| {
            Ok(Layer {
                id: row.get(0)?,
                name: row.get(1)?,
                display_order: row.get(2)?,
                filter_json: row.get(3)?,
                metadata: row.get(4)?,
                created_at: row.get(5)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layers)
}

pub fn query_world_config(conn: &Connection) -> Result<Option<WorldConfig>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, config_json, created_at FROM worlds ORDER BY created_at DESC LIMIT 1",
    )?;
    let mut rows = stmt.query([])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    Ok(Some(WorldConfig {
        id: row.get(0)?,
        name: row.get(1)?,
        config_json: row.get(2)?,
        created_at: row.get(3)?,
    }))
}

pub fn query_relation_kinds(conn: &Connection) -> Result<Vec<RelationKind>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, world_id, label, directed, default_weight, metadata, created_at
         FROM relation_kinds
         ORDER BY label ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(RelationKind {
                id: row.get(0)?,
                world_id: row.get(1)?,
                label: row.get(2)?,
                directed: row.get::<_, i32>(3)? != 0,
                default_weight: row.get(4)?,
                metadata: row.get(5)?,
                created_at: row.get(6)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn query_connection_layers(conn: &Connection) -> Result<Vec<ConnectionLayer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, display_order, metadata, created_at
         FROM connection_layers
         ORDER BY display_order ASC, name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(ConnectionLayer {
                id: row.get(0)?,
                name: row.get(1)?,
                display_order: row.get(2)?,
                metadata: row.get(3)?,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn insert_layer(conn: &Connection, name: &str, display_order: i32) -> Result<Layer, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    conn.execute(
        "INSERT INTO layers (id, name, display_order, filter_json, metadata, created_at)
         VALUES (?1, ?2, ?3, '{}', '{}', ?4)",
        params![id, name, display_order, created_at],
    )?;
    Ok(Layer {
        id,
        name: name.to_string(),
        display_order,
        filter_json: "{}".to_string(),
        metadata: "{}".to_string(),
        created_at,
    })
}

// ---------------------------------------------------------------------------
// Edge helpers
// ---------------------------------------------------------------------------

fn query_edge_connection_layer_ids(
    conn: &Connection,
    edge_id: &str,
) -> Result<Vec<String>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT connection_layer_id
         FROM edge_connection_layers
         WHERE edge_id = ?1",
    )?;
    let ids = stmt
        .query_map([edge_id], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ids)
}

fn query_edges_for_node(conn: &Connection, node_id: &str) -> Result<Vec<EdgeRef>, AppError> {
    // Include both outgoing (source=node) and incoming (target=node) edges.
    // target_id is normalised to always mean "the other end of this edge".
    let mut stmt = conn.prepare(
        "SELECT id,
                CASE WHEN source_id = ?1 THEN target_id ELSE source_id END AS neighbor_id,
                edge_type,
                relation_id,
                weight
         FROM edges
         WHERE source_id = ?1 OR target_id = ?1",
    )?;
    struct EdgeRow {
        id: String,
        target_id: String,
        edge_type: String,
        relation_id: Option<String>,
        weight: f64,
    }

    let rows = stmt
        .query_map([node_id], |row| {
            Ok(EdgeRow {
                id: row.get(0)?,
                target_id: row.get(1)?,
                edge_type: row.get(2)?,
                relation_id: row.get(3)?,
                weight: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut edges = Vec::with_capacity(rows.len());
    for row in rows {
        edges.push(EdgeRef {
            id: row.id.clone(),
            target_id: row.target_id,
            edge_type: row.edge_type,
            relation_id: row.relation_id,
            connection_layer_ids: query_edge_connection_layer_ids(conn, &row.id)?,
            weight: row.weight,
        });
    }

    Ok(edges)
}

// ---------------------------------------------------------------------------
// Node operations
// ---------------------------------------------------------------------------

pub fn query_nodes(conn: &Connection, layer_id: &str) -> Result<Vec<Node>, AppError> {
    struct NodeRow {
        id: String,
        title: String,
        layer_id: String,
        node_type: String,
        note_type_id: Option<String>,
        note_fields_json: String,
        content_type: String,
        content_data: Option<String>,
        tags_json: String,
        learned: bool,
        weight: f64,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        pos_z: Option<f64>,
        created_at: String,
    }

    let mut stmt = conn.prepare(
        "SELECT id, title, layer_id, node_type, note_type_id, note_fields, content_type, content_data,
                 tags, learned, weight, pos_x, pos_y, pos_z, created_at
         FROM nodes n
         WHERE n.layer_id = ?1
            OR EXISTS (
              SELECT 1
              FROM node_layers nl
              WHERE nl.node_id = n.id AND nl.layer_id = ?1
            )",
    )?;

    let rows: Vec<NodeRow> = stmt
        .query_map([layer_id], |row| {
            Ok(NodeRow {
                id: row.get(0)?,
                title: row.get(1)?,
                layer_id: row.get(2)?,
                node_type: row.get(3)?,
                note_type_id: row.get(4)?,
                note_fields_json: row.get(5)?,
                content_type: row.get(6)?,
                content_data: row.get(7)?,
                tags_json: row.get(8)?,
                learned: row.get::<_, i32>(9)? != 0,
                weight: row.get(10)?,
                pos_x: row.get(11)?,
                pos_y: row.get(12)?,
                pos_z: row.get(13)?,
                created_at: row.get(14)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut nodes = Vec::new();
    for row in rows {
        let tags: Vec<String> = serde_json::from_str(&row.tags_json).unwrap_or_default();
        let note_fields: BTreeMap<String, String> =
            serde_json::from_str(&row.note_fields_json).unwrap_or_default();
        let connections = query_edges_for_node(conn, &row.id)?;
        nodes.push(Node {
            id: row.id,
            title: row.title,
            layer_id: row.layer_id,
            node_type: row.node_type,
            note_type_id: row.note_type_id,
            note_fields,
            content_type: row.content_type,
            content_data: row.content_data,
            tags,
            learned: row.learned,
            weight: row.weight,
            pos_x: row.pos_x,
            pos_y: row.pos_y,
            pos_z: row.pos_z,
            connections,
            created_at: row.created_at,
        });
    }
    Ok(nodes)
}

pub fn insert_node(conn: &Connection, input: CreateNodeInput) -> Result<Node, AppError> {
    let CreateNodeInput {
        title,
        layer_id,
        node_type,
        note_type_id,
        note_fields,
        content_data,
        tags,
        weight,
    } = input;

    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    let tags_json = serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string());
    let note_fields = note_fields.unwrap_or_default();
    let note_fields_json = serde_json::to_string(&note_fields).unwrap_or_else(|_| "{}".to_string());
    let note_type_id = note_type_id.or_else(|| default_note_type_id(conn).ok().flatten());

    conn.execute(
        "INSERT INTO nodes
             (id, title, layer_id, node_type, note_type_id, note_fields, content_type, content_data, tags, learned, weight, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'text', ?7, ?8, 0, ?9, ?10)",
        params![
            id,
            title,
            layer_id,
            node_type,
            note_type_id,
            note_fields_json,
            content_data,
            tags_json,
            weight,
            created_at
        ],
    )?;

    let _ = conn.execute(
        "INSERT OR IGNORE INTO node_layers (node_id, layer_id, created_at) VALUES (?1, ?2, ?3)",
        params![id, layer_id, now_ts()],
    );

    Ok(Node {
        id,
        title,
        layer_id,
        node_type,
        note_type_id,
        note_fields,
        content_type: "text".to_string(),
        content_data,
        tags,
        learned: false,
        weight,
        pos_x: None,
        pos_y: None,
        pos_z: None,
        connections: vec![],
        created_at,
    })
}

pub fn set_learned(conn: &Connection, id: &str, learned: bool) -> Result<Node, AppError> {
    let learned_int = if learned { 1i32 } else { 0i32 };
    let changed = conn.execute(
        "UPDATE nodes SET learned = ?1 WHERE id = ?2",
        params![learned_int, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("Node {} not found", id)));
    }
    query_single_node(conn, id)
}

pub fn set_node_position(
    conn: &Connection,
    id: &str,
    x: f64,
    y: f64,
    z: f64,
) -> Result<(), AppError> {
    let changed = conn.execute(
        "UPDATE nodes SET pos_x = ?1, pos_y = ?2, pos_z = ?3 WHERE id = ?4",
        params![x, y, z, id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("Node {} not found", id)));
    }
    Ok(())
}

fn query_single_node(conn: &Connection, id: &str) -> Result<Node, AppError> {
    struct NodeData {
        title: String,
        layer_id: String,
        node_type: String,
        note_type_id: Option<String>,
        note_fields_json: String,
        content_type: String,
        content_data: Option<String>,
        tags_json: String,
        learned: bool,
        weight: f64,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        pos_z: Option<f64>,
        created_at: String,
    }

    let data = conn
        .query_row(
            "SELECT title, layer_id, node_type, note_type_id, note_fields, content_type, content_data,
                    tags, learned, weight, pos_x, pos_y, pos_z, created_at
             FROM nodes WHERE id = ?1",
            [id],
            |row| {
                Ok(NodeData {
                    title: row.get(0)?,
                    layer_id: row.get(1)?,
                    node_type: row.get(2)?,
                    note_type_id: row.get(3)?,
                    note_fields_json: row.get(4)?,
                    content_type: row.get(5)?,
                    content_data: row.get(6)?,
                    tags_json: row.get(7)?,
                    learned: row.get::<_, i32>(8)? != 0,
                    weight: row.get(9)?,
                    pos_x: row.get(10)?,
                    pos_y: row.get(11)?,
                    pos_z: row.get(12)?,
                    created_at: row.get(13)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                AppError::NotFound(format!("Node {} not found", id))
            }
            other => AppError::Database(other),
        })?;

    let tags: Vec<String> = serde_json::from_str(&data.tags_json).unwrap_or_default();
    let note_fields: BTreeMap<String, String> =
        serde_json::from_str(&data.note_fields_json).unwrap_or_default();
    let connections = query_edges_for_node(conn, id)?;

    Ok(Node {
        id: id.to_string(),
        title: data.title,
        layer_id: data.layer_id,
        node_type: data.node_type,
        note_type_id: data.note_type_id,
        note_fields,
        content_type: data.content_type,
        content_data: data.content_data,
        tags,
        learned: data.learned,
        weight: data.weight,
        pos_x: data.pos_x,
        pos_y: data.pos_y,
        pos_z: data.pos_z,
        connections,
        created_at: data.created_at,
    })
}

// ---------------------------------------------------------------------------
// Edge operations
// ---------------------------------------------------------------------------

pub fn insert_edge(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    edge_type: &str,
) -> Result<Edge, AppError> {
    insert_edge_with_relation(conn, source_id, target_id, edge_type, None)
}

pub fn insert_edge_with_relation(
    conn: &Connection,
    source_id: &str,
    target_id: &str,
    edge_type: &str,
    relation_id: Option<&str>,
) -> Result<Edge, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    let weight = 1.0_f64;
    conn.execute(
        "INSERT INTO edges (id, source_id, target_id, edge_type, relation_id, weight, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            id,
            source_id,
            target_id,
            edge_type,
            relation_id,
            weight,
            created_at
        ],
    )?;

    if let Ok(source_layer) = conn.query_row(
        "SELECT layer_id FROM nodes WHERE id = ?1",
        [source_id],
        |row| row.get::<_, String>(0),
    ) {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO edge_layers (edge_id, layer_id, created_at) VALUES (?1, ?2, ?3)",
            params![id, source_layer, now_ts()],
        );
    }

    if let Ok(default_connection_layer_id) = conn.query_row(
        "SELECT id FROM connection_layers ORDER BY display_order ASC, name ASC LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    ) {
        let _ = conn.execute(
            "INSERT OR IGNORE INTO edge_connection_layers (edge_id, connection_layer_id, created_at) VALUES (?1, ?2, ?3)",
            params![id, default_connection_layer_id, now_ts()],
        );
    }

    Ok(Edge {
        id,
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        edge_type: edge_type.to_string(),
        relation_id: relation_id.map(str::to_string),
        weight,
        created_at,
    })
}

pub fn remove_edge(conn: &Connection, id: &str) -> Result<(), AppError> {
    let changed = conn.execute("DELETE FROM edges WHERE id = ?1", [id])?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("Edge {} not found", id)));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Seed / reset helpers — delegate to the domain pack loader
// ---------------------------------------------------------------------------

pub fn seed_sample_data(conn: &Connection) -> Result<(), AppError> {
    ensure_default_note_types(conn)?;
    // One-time migration: remove the old hardcoded "Japanese N5 Grammar" layer
    // (and its nodes/edges via CASCADE) that existed before the domain-pack system.
    // Safe to call every launch — after the first run there is nothing to delete.
    conn.execute("DELETE FROM layers WHERE name = 'Japanese N5 Grammar'", [])?;

    // The Japanese N5 starter pack is embedded at compile time.
    // To add a new domain: create domains/<name>/pack.json and call domain::seed_pack here.
    let json = include_str!("../../domains/japanese/pack.json");
    crate::domain::seed_pack(conn, json)?;
    reconcile_duplicate_layers(conn)?;
    Ok(())
}

/// Wipe all graph data and re-seed from the bundled domain pack.
/// Use during development when seed data changes between runs.
pub fn reset_and_reseed(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        "DELETE FROM edge_connection_layers;
         DELETE FROM connection_layers;
         DELETE FROM edges;
         DELETE FROM node_layers;
         DELETE FROM nodes;
         DELETE FROM layers;
         DELETE FROM relation_kinds;
         DELETE FROM worlds;
         DELETE FROM note_types;",
    )?;
    seed_sample_data(conn)
}

fn reconcile_duplicate_layers(conn: &Connection) -> Result<(), AppError> {
    let mut stmt = conn.prepare("SELECT name FROM layers GROUP BY name HAVING COUNT(*) > 1")?;
    let duplicate_names = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for name in duplicate_names {
        let mut ids_stmt =
            conn.prepare("SELECT id FROM layers WHERE name = ?1 ORDER BY created_at ASC, id ASC")?;
        let layer_ids = ids_stmt
            .query_map([&name], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        if layer_ids.len() < 2 {
            continue;
        }

        let keeper_id = &layer_ids[0];
        for dup_id in layer_ids.iter().skip(1) {
            conn.execute(
                "UPDATE nodes SET layer_id = ?1 WHERE layer_id = ?2",
                params![keeper_id, dup_id],
            )?;
            conn.execute(
                "INSERT OR IGNORE INTO node_layers (node_id, layer_id, created_at)
                 SELECT node_id, ?1, created_at FROM node_layers WHERE layer_id = ?2",
                params![keeper_id, dup_id],
            )?;
            conn.execute("DELETE FROM node_layers WHERE layer_id = ?1", [dup_id])?;

            conn.execute(
                "INSERT OR IGNORE INTO edge_layers (edge_id, layer_id, created_at)
                 SELECT edge_id, ?1, created_at FROM edge_layers WHERE layer_id = ?2",
                params![keeper_id, dup_id],
            )?;
            conn.execute("DELETE FROM edge_layers WHERE layer_id = ?1", [dup_id])?;

            conn.execute("DELETE FROM layers WHERE id = ?1", [dup_id])?;
        }
    }

    Ok(())
}

pub fn query_note_types(conn: &Connection) -> Result<Vec<NoteType>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, fields, is_default, created_at
         FROM note_types
         ORDER BY is_default DESC, name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            let fields_json: String = row.get(2)?;
            let fields: Vec<String> = serde_json::from_str(&fields_json).unwrap_or_default();
            Ok(NoteType {
                id: row.get(0)?,
                name: row.get(1)?,
                fields,
                is_default: row.get::<_, i32>(3)? != 0,
                created_at: row.get(4)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn insert_note_type(
    conn: &Connection,
    name: &str,
    fields: Vec<String>,
    is_default: bool,
) -> Result<NoteType, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    let fields_json = serde_json::to_string(&fields).unwrap_or_else(|_| "[]".to_string());
    let is_default_i = if is_default { 1 } else { 0 };
    if is_default {
        conn.execute("UPDATE note_types SET is_default = 0", [])?;
    }
    conn.execute(
        "INSERT INTO note_types (id, name, fields, is_default, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![id, name, fields_json, is_default_i, created_at],
    )?;
    Ok(NoteType {
        id,
        name: name.to_string(),
        fields,
        is_default,
        created_at,
    })
}

pub fn set_node_note_type(
    conn: &Connection,
    node_id: &str,
    note_type_id: Option<String>,
) -> Result<Node, AppError> {
    let changed = conn.execute(
        "UPDATE nodes SET note_type_id = ?1 WHERE id = ?2",
        params![note_type_id, node_id],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("Node {} not found", node_id)));
    }
    query_single_node(conn, node_id)
}

fn default_note_type_id(conn: &Connection) -> Result<Option<String>, AppError> {
    let res = conn.query_row(
        "SELECT id FROM note_types WHERE is_default = 1 LIMIT 1",
        [],
        |row| row.get::<_, String>(0),
    );
    match res {
        Ok(id) => Ok(Some(id)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(AppError::Database(e)),
    }
}

fn ensure_default_note_types(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM note_types", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    insert_note_type(conn, "Basic", vec!["Front".into(), "Back".into()], true)?;
    insert_note_type(conn, "Cloze", vec!["Text".into(), "Extra".into()], false)?;
    insert_note_type(
        conn,
        "Vocab",
        vec![
            "Word".into(),
            "Reading".into(),
            "Meaning".into(),
            "Example".into(),
        ],
        false,
    )?;
    Ok(())
}
