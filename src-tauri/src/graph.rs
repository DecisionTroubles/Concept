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
pub struct NodeProgress {
    pub node_id: String,
    pub status: String,
    pub review_count: i32,
    pub streak: i32,
    pub last_reviewed_at: Option<String>,
    pub next_review_at: Option<String>,
    pub scheduler_key: String,
    pub scheduler_state: String,
    pub created_at: String,
    pub updated_at: String,
}

#[taurpc::ipc_type]
pub struct Node {
    pub id: String,
    pub title: String,
    pub layer_id: String,
    pub parent_node_id: Option<String>,
    pub node_type: String,
    pub note_type_id: Option<String>,
    pub note_fields: BTreeMap<String, String>,
    pub content_type: String,
    pub content_data: Option<String>,
    pub tags: Vec<String>,
    pub learned: bool,
    pub progress_status: String,
    pub progress_review_count: i32,
    pub progress_streak: i32,
    pub progress_last_reviewed_at: Option<String>,
    pub progress_next_review_at: Option<String>,
    pub progress_scheduler_key: String,
    pub progress_scheduler_state: String,
    pub weight: f64,
    pub pos_x: Option<f64>,
    pub pos_y: Option<f64>,
    pub pos_z: Option<f64>,
    pub metadata: String,
    /// Outgoing edges — loaded in the same call, no second IPC round-trip needed.
    pub connections: Vec<EdgeRef>,
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct CreateNodeInput {
    pub title: String,
    pub layer_id: String,
    pub parent_node_id: Option<String>,
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
    pub world_id: Option<String>,
    pub base_note_type_id: Option<String>,
    pub fields: Vec<String>,
    pub schema_json: String,
    pub layout_json: String,
    pub metadata: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[taurpc::ipc_type]
pub struct NoteTypeInput {
    pub name: String,
    pub world_id: Option<String>,
    pub base_note_type_id: Option<String>,
    pub fields: Vec<String>,
    pub schema_json: String,
    pub layout_json: String,
    pub metadata: String,
    pub is_default: bool,
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
        parent_node_id: Option<String>,
        node_type: String,
        note_type_id: Option<String>,
        note_fields_json: String,
        content_type: String,
        content_data: Option<String>,
        tags_json: String,
        learned: bool,
        progress_status: String,
        progress_review_count: i32,
        progress_streak: i32,
        progress_last_reviewed_at: Option<String>,
        progress_next_review_at: Option<String>,
        progress_scheduler_key: String,
        progress_scheduler_state: String,
        weight: f64,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        pos_z: Option<f64>,
        metadata: String,
        created_at: String,
    }

    let mut stmt = conn.prepare(
        "SELECT n.id, n.title, n.layer_id, n.parent_node_id, n.node_type, n.note_type_id, n.note_fields, n.content_type, n.content_data,
                 n.tags, n.learned,
                 COALESCE(np.status, 'new'),
                 COALESCE(np.review_count, 0),
                 COALESCE(np.streak, 0),
                 np.last_reviewed_at,
                 np.next_review_at,
                 COALESCE(np.scheduler_key, 'basic-v1'),
                 COALESCE(np.scheduler_state, '{}'),
                 n.weight, n.pos_x, n.pos_y, n.pos_z, n.metadata, n.created_at
         FROM nodes n
         LEFT JOIN node_progress np ON np.node_id = n.id
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
                parent_node_id: row.get(3)?,
                node_type: row.get(4)?,
                note_type_id: row.get(5)?,
                note_fields_json: row.get(6)?,
                content_type: row.get(7)?,
                content_data: row.get(8)?,
                tags_json: row.get(9)?,
                learned: row.get::<_, i32>(10)? != 0,
                progress_status: row.get(11)?,
                progress_review_count: row.get(12)?,
                progress_streak: row.get(13)?,
                progress_last_reviewed_at: row.get(14)?,
                progress_next_review_at: row.get(15)?,
                progress_scheduler_key: row.get(16)?,
                progress_scheduler_state: row.get(17)?,
                weight: row.get(18)?,
                pos_x: row.get(19)?,
                pos_y: row.get(20)?,
                pos_z: row.get(21)?,
                metadata: row.get(22)?,
                created_at: row.get(23)?,
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
            parent_node_id: row.parent_node_id,
            node_type: row.node_type,
            note_type_id: row.note_type_id,
            note_fields,
            content_type: row.content_type,
            content_data: row.content_data,
            tags,
            learned: row.learned,
            progress_status: row.progress_status,
            progress_review_count: row.progress_review_count,
            progress_streak: row.progress_streak,
            progress_last_reviewed_at: row.progress_last_reviewed_at,
            progress_next_review_at: row.progress_next_review_at,
            progress_scheduler_key: row.progress_scheduler_key,
            progress_scheduler_state: row.progress_scheduler_state,
            weight: row.weight,
            pos_x: row.pos_x,
            pos_y: row.pos_y,
            pos_z: row.pos_z,
            metadata: row.metadata,
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
        parent_node_id,
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
             (id, title, layer_id, parent_node_id, node_type, note_type_id, note_fields, content_type, content_data, tags, learned, weight, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, 'text', ?8, ?9, 0, ?10, ?11)",
        params![
            id,
            title,
            layer_id,
            parent_node_id,
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

    let progress_created_at = now_ts();
    let _ = conn.execute(
        "INSERT OR IGNORE INTO node_progress
            (node_id, status, review_count, streak, last_reviewed_at, next_review_at, scheduler_key, scheduler_state, created_at, updated_at)
         VALUES (?1, 'new', 0, 0, NULL, NULL, 'basic-v1', '{}', ?2, ?2)",
        params![id, progress_created_at],
    );

    Ok(Node {
        id,
        title,
        layer_id,
        parent_node_id,
        node_type,
        note_type_id,
        note_fields,
        content_type: "text".to_string(),
        content_data,
        tags,
        learned: false,
        progress_status: "new".to_string(),
        progress_review_count: 0,
        progress_streak: 0,
        progress_last_reviewed_at: None,
        progress_next_review_at: None,
        progress_scheduler_key: "basic-v1".to_string(),
        progress_scheduler_state: "{}".to_string(),
        weight,
        pos_x: None,
        pos_y: None,
        pos_z: None,
        metadata: "{}".to_string(),
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
    let updated_at = now_ts();
    let status = if learned { "mastered" } else { "new" };
    let _ = conn.execute(
        "INSERT INTO node_progress
            (node_id, status, review_count, streak, last_reviewed_at, next_review_at, scheduler_key, scheduler_state, created_at, updated_at)
         VALUES (?1, ?2, 0, 0, NULL, NULL, 'basic-v1', '{}', ?3, ?3)
         ON CONFLICT(node_id) DO UPDATE SET
            status = excluded.status,
            scheduler_key = excluded.scheduler_key,
            scheduler_state = excluded.scheduler_state,
            updated_at = excluded.updated_at",
        params![id, status, updated_at],
    );
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

pub fn query_single_node(conn: &Connection, id: &str) -> Result<Node, AppError> {
    struct NodeData {
        title: String,
        layer_id: String,
        parent_node_id: Option<String>,
        node_type: String,
        note_type_id: Option<String>,
        note_fields_json: String,
        content_type: String,
        content_data: Option<String>,
        tags_json: String,
        learned: bool,
        progress_status: String,
        progress_review_count: i32,
        progress_streak: i32,
        progress_last_reviewed_at: Option<String>,
        progress_next_review_at: Option<String>,
        progress_scheduler_key: String,
        progress_scheduler_state: String,
        weight: f64,
        pos_x: Option<f64>,
        pos_y: Option<f64>,
        pos_z: Option<f64>,
        metadata: String,
        created_at: String,
    }

    let data = conn
        .query_row(
            "SELECT n.title, n.layer_id, n.parent_node_id, n.node_type, n.note_type_id, n.note_fields, n.content_type, n.content_data,
                    n.tags, n.learned,
                    COALESCE(np.status, 'new'),
                    COALESCE(np.review_count, 0),
                    COALESCE(np.streak, 0),
                    np.last_reviewed_at,
                    np.next_review_at,
                    COALESCE(np.scheduler_key, 'basic-v1'),
                    COALESCE(np.scheduler_state, '{}'),
                    n.weight, n.pos_x, n.pos_y, n.pos_z, n.metadata, n.created_at
             FROM nodes n
             LEFT JOIN node_progress np ON np.node_id = n.id
             WHERE n.id = ?1",
            [id],
            |row| {
                Ok(NodeData {
                    title: row.get(0)?,
                    layer_id: row.get(1)?,
                    parent_node_id: row.get(2)?,
                    node_type: row.get(3)?,
                    note_type_id: row.get(4)?,
                    note_fields_json: row.get(5)?,
                    content_type: row.get(6)?,
                    content_data: row.get(7)?,
                    tags_json: row.get(8)?,
                    learned: row.get::<_, i32>(9)? != 0,
                    progress_status: row.get(10)?,
                    progress_review_count: row.get(11)?,
                    progress_streak: row.get(12)?,
                    progress_last_reviewed_at: row.get(13)?,
                    progress_next_review_at: row.get(14)?,
                    progress_scheduler_key: row.get(15)?,
                    progress_scheduler_state: row.get(16)?,
                    weight: row.get(17)?,
                    pos_x: row.get(18)?,
                    pos_y: row.get(19)?,
                    pos_z: row.get(20)?,
                    metadata: row.get(21)?,
                    created_at: row.get(22)?,
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
        parent_node_id: data.parent_node_id,
        node_type: data.node_type,
        note_type_id: data.note_type_id,
        note_fields,
        content_type: data.content_type,
        content_data: data.content_data,
        tags,
        learned: data.learned,
        progress_status: data.progress_status,
        progress_review_count: data.progress_review_count,
        progress_streak: data.progress_streak,
        progress_last_reviewed_at: data.progress_last_reviewed_at,
        progress_next_review_at: data.progress_next_review_at,
        progress_scheduler_key: data.progress_scheduler_key,
        progress_scheduler_state: data.progress_scheduler_state,
        weight: data.weight,
        pos_x: data.pos_x,
        pos_y: data.pos_y,
        pos_z: data.pos_z,
        metadata: data.metadata,
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
    crate::world_registry::reload_active_world(conn)
}

pub fn reset_data(conn: &Connection, reseed: bool) -> Result<(), AppError> {
    conn.execute_batch(
        "DELETE FROM edge_connection_layers;
         DELETE FROM connection_layers;
         DELETE FROM edges;
         DELETE FROM node_extension_data;
         DELETE FROM review_events;
         DELETE FROM node_progress;
         DELETE FROM node_layers;
         DELETE FROM nodes;
         DELETE FROM layers;
         DELETE FROM relation_kinds;
         DELETE FROM worlds;
         DELETE FROM note_types;",
    )?;
    if reseed {
        crate::world_registry::reload_active_world(conn)?;
    }
    Ok(())
}

pub fn query_node_progress(conn: &Connection) -> Result<Vec<NodeProgress>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT node_id, status, review_count, streak, last_reviewed_at, next_review_at, scheduler_key, scheduler_state, created_at, updated_at
         FROM node_progress
         ORDER BY updated_at DESC, node_id ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            Ok(NodeProgress {
                node_id: row.get(0)?,
                status: row.get(1)?,
                review_count: row.get(2)?,
                streak: row.get(3)?,
                last_reviewed_at: row.get(4)?,
                next_review_at: row.get(5)?,
                scheduler_key: row.get(6)?,
                scheduler_state: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

pub fn set_node_progress_status(conn: &Connection, node_id: &str, status: &str) -> Result<Node, AppError> {
    let updated_at = now_ts();
    let next_review_at = if status == "mastered" {
        None::<String>
    } else {
        Some(updated_at.clone())
    };
    let review_increment = matches!(status, "review" | "mastered") as i32;
    let streak_increment = matches!(status, "review" | "mastered") as i32;
    conn.execute(
        "INSERT INTO node_progress
            (node_id, status, review_count, streak, last_reviewed_at, next_review_at, scheduler_key, scheduler_state, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'basic-v1', '{}', ?7, ?7)
         ON CONFLICT(node_id) DO UPDATE SET
            status = excluded.status,
            review_count = CASE
                WHEN excluded.review_count > 0 THEN node_progress.review_count + excluded.review_count
                ELSE node_progress.review_count
            END,
            streak = CASE
                WHEN excluded.status = 'new' THEN 0
                WHEN excluded.status = 'learning' THEN MAX(node_progress.streak, 0)
                WHEN excluded.status = 'review' OR excluded.status = 'mastered' THEN node_progress.streak + excluded.streak
                ELSE node_progress.streak
            END,
            last_reviewed_at = excluded.last_reviewed_at,
            next_review_at = excluded.next_review_at,
            scheduler_key = excluded.scheduler_key,
            scheduler_state = excluded.scheduler_state,
            updated_at = excluded.updated_at",
        params![
            node_id,
            status,
            review_increment,
            streak_increment,
            Some(updated_at.clone()),
            next_review_at,
            updated_at
        ],
    )?;
    let learned = status == "mastered";
    let _ = conn.execute(
        "UPDATE nodes SET learned = ?1 WHERE id = ?2",
        params![if learned { 1 } else { 0 }, node_id],
    );
    query_single_node(conn, node_id)
}

pub(crate) fn reconcile_duplicate_layers(conn: &Connection) -> Result<(), AppError> {
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
        "SELECT id, name, world_id, base_note_type_id, fields, schema_json, layout_json, metadata, is_default, created_at, updated_at
         FROM note_types
         ORDER BY CASE WHEN world_id IS NULL THEN 0 ELSE 1 END ASC, is_default DESC, name ASC",
    )?;
    let rows = stmt
        .query_map([], |row| {
            let fields_json: String = row.get(4)?;
            let fields: Vec<String> = serde_json::from_str(&fields_json).unwrap_or_default();
            Ok(NoteType {
                id: row.get(0)?,
                name: row.get(1)?,
                world_id: row.get(2)?,
                base_note_type_id: row.get(3)?,
                fields,
                schema_json: row.get(5)?,
                layout_json: row.get(6)?,
                metadata: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(rows)
}

fn insert_note_type_with_id(
    conn: &Connection,
    id: &str,
    input: NoteTypeInput,
) -> Result<NoteType, AppError> {
    let created_at = now_ts();
    let updated_at = created_at.clone();
    let fields_json = serde_json::to_string(&input.fields).unwrap_or_else(|_| "[]".to_string());
    let schema_json = if input.schema_json.trim().is_empty() {
        default_note_type_schema_json(&input.name, &input.fields)
    } else {
        input.schema_json
    };
    let layout_json = if input.layout_json.trim().is_empty() {
        default_note_type_layout_json(&input.name, &input.fields)
    } else {
        input.layout_json
    };
    let metadata = if input.metadata.trim().is_empty() {
        "{}".to_string()
    } else {
        input.metadata
    };
    let is_default_i = if input.is_default { 1 } else { 0 };
    if input.is_default {
        conn.execute("UPDATE note_types SET is_default = 0 WHERE world_id IS ?1", [input.world_id.clone()])?;
    }
    conn.execute(
        "INSERT INTO note_types
            (id, name, world_id, base_note_type_id, fields, schema_json, layout_json, metadata, is_default, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![
            id,
            input.name,
            input.world_id,
            input.base_note_type_id,
            fields_json,
            schema_json,
            layout_json,
            metadata,
            is_default_i,
            created_at,
            updated_at
        ],
    )?;
    query_note_type(conn, id)
}

pub fn query_note_type(conn: &Connection, id: &str) -> Result<NoteType, AppError> {
    conn.query_row(
        "SELECT id, name, world_id, base_note_type_id, fields, schema_json, layout_json, metadata, is_default, created_at, updated_at
         FROM note_types WHERE id = ?1",
        [id],
        |row| {
            let fields_json: String = row.get(4)?;
            let fields: Vec<String> = serde_json::from_str(&fields_json).unwrap_or_default();
            Ok(NoteType {
                id: row.get(0)?,
                name: row.get(1)?,
                world_id: row.get(2)?,
                base_note_type_id: row.get(3)?,
                fields,
                schema_json: row.get(5)?,
                layout_json: row.get(6)?,
                metadata: row.get(7)?,
                is_default: row.get::<_, i32>(8)? != 0,
                created_at: row.get(9)?,
                updated_at: row.get(10)?,
            })
        },
    ).map_err(|e| match e {
        rusqlite::Error::QueryReturnedNoRows => AppError::NotFound(format!("Note type {} not found", id)),
        other => AppError::Database(other),
    })
}

pub fn insert_note_type(conn: &Connection, input: NoteTypeInput) -> Result<NoteType, AppError> {
    let id = Uuid::new_v4().to_string();
    insert_note_type_with_id(conn, &id, input)
}

pub fn update_note_type(conn: &Connection, id: &str, input: NoteTypeInput) -> Result<NoteType, AppError> {
    let fields_json = serde_json::to_string(&input.fields).unwrap_or_else(|_| "[]".to_string());
    let schema_json = if input.schema_json.trim().is_empty() {
        default_note_type_schema_json(&input.name, &input.fields)
    } else {
        input.schema_json
    };
    let layout_json = if input.layout_json.trim().is_empty() {
        default_note_type_layout_json(&input.name, &input.fields)
    } else {
        input.layout_json
    };
    let metadata = if input.metadata.trim().is_empty() {
        "{}".to_string()
    } else {
        input.metadata
    };
    if input.is_default {
        conn.execute("UPDATE note_types SET is_default = 0 WHERE world_id IS ?1", [input.world_id.clone()])?;
    }
    let changed = conn.execute(
        "UPDATE note_types
         SET name = ?1, world_id = ?2, base_note_type_id = ?3, fields = ?4, schema_json = ?5, layout_json = ?6, metadata = ?7, is_default = ?8, updated_at = ?9
         WHERE id = ?10",
        params![
            input.name,
            input.world_id,
            input.base_note_type_id,
            fields_json,
            schema_json,
            layout_json,
            metadata,
            if input.is_default { 1 } else { 0 },
            now_ts(),
            id
        ],
    )?;
    if changed == 0 {
        return Err(AppError::NotFound(format!("Note type {} not found", id)));
    }
    query_note_type(conn, id)
}

pub fn duplicate_note_type(
    conn: &Connection,
    source_id: &str,
    name: &str,
    world_id: Option<String>,
) -> Result<NoteType, AppError> {
    let source = query_note_type(conn, source_id)?;
    insert_note_type(conn, NoteTypeInput {
        name: name.to_string(),
        world_id,
        base_note_type_id: Some(source.id.clone()),
        fields: source.fields,
        schema_json: source.schema_json,
        layout_json: source.layout_json,
        metadata: source.metadata,
        is_default: false,
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

pub fn update_node_content(
    conn: &Connection,
    node_id: &str,
    title: String,
    note_fields: BTreeMap<String, String>,
    content_data: Option<String>,
    tags: Vec<String>,
) -> Result<Node, AppError> {
    let changed = conn.execute(
        "UPDATE nodes
         SET title = ?1, note_fields = ?2, content_data = ?3, tags = ?4
         WHERE id = ?5",
        params![
            title,
            serde_json::to_string(&note_fields).unwrap_or_else(|_| "{}".to_string()),
            content_data,
            serde_json::to_string(&tags).unwrap_or_else(|_| "[]".to_string()),
            node_id
        ],
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

pub(crate) fn ensure_default_note_types(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM note_types", [], |row| row.get(0))?;
    if count > 0 {
        return Ok(());
    }

    insert_note_type_with_id(conn, "basic", NoteTypeInput {
        name: "Basic".into(),
        world_id: None,
        base_note_type_id: None,
        fields: vec!["Front".into(), "Back".into()],
        schema_json: String::new(),
        layout_json: String::new(),
        metadata: "{}".into(),
        is_default: true,
    })?;
    insert_note_type_with_id(conn, "cloze", NoteTypeInput {
        name: "Cloze".into(),
        world_id: None,
        base_note_type_id: None,
        fields: vec!["Text".into(), "Extra".into()],
        schema_json: String::new(),
        layout_json: String::new(),
        metadata: "{}".into(),
        is_default: false,
    })?;
    insert_note_type_with_id(conn, "vocab", NoteTypeInput {
        name: "Vocab".into(),
        world_id: None,
        base_note_type_id: None,
        fields: vec!["Word".into(), "Reading".into(), "Meaning".into(), "Example".into()],
        schema_json: String::new(),
        layout_json: String::new(),
        metadata: "{}".into(),
        is_default: false,
    })?;
    Ok(())
}

fn default_note_type_schema_json(name: &str, fields: &[String]) -> String {
    let defs = fields
        .iter()
        .map(|field| {
            let widget = if field.eq_ignore_ascii_case("example") || field.eq_ignore_ascii_case("extra") {
                "long_text"
            } else {
                "text"
            };
            serde_json::json!({
                "key": field,
                "label": field,
                "type": "string",
                "widget": widget,
            })
        })
        .collect::<Vec<_>>();

    serde_json::json!({
        "version": 1,
        "name": name,
        "fields": defs,
    })
    .to_string()
}

fn default_note_type_layout_json(name: &str, fields: &[String]) -> String {
    let primary_fields = if fields.is_empty() {
        vec![serde_json::json!({ "field": "Content" })]
    } else {
        fields
            .iter()
            .map(|field| serde_json::json!({ "field": field }))
            .collect::<Vec<_>>()
    };

    serde_json::json!({
        "version": 1,
        "pages": [
            {
                "id": "content",
                "kind": "content",
                "label": format!("{} Content", name),
                "sections": [
                    {
                        "id": "main",
                        "label": "Main",
                        "items": primary_fields
                    }
                ]
            },
            {
                "id": "connections",
                "kind": "built_in",
                "label": "Connections",
                "source": "connections"
            },
            {
                "id": "learning",
                "kind": "built_in",
                "label": "Learning",
                "source": "learning"
            },
            {
                "id": "history",
                "kind": "built_in",
                "label": "History",
                "source": "history"
            }
        ]
    })
    .to_string()
}
