use rusqlite::{params, Connection};
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
    pub created_at: String,
}

#[taurpc::ipc_type]
pub struct EdgeRef {
    pub id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f64,
}

#[taurpc::ipc_type]
pub struct Node {
    pub id: String,
    pub title: String,
    pub layer_id: String,
    pub node_type: String,
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
    pub content_data: Option<String>,
    pub tags: Vec<String>,
    pub weight: f64,
}

#[taurpc::ipc_type]
pub struct Edge {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub edge_type: String,
    pub weight: f64,
    pub created_at: String,
}

// ---------------------------------------------------------------------------
// Layer operations
// ---------------------------------------------------------------------------

pub fn query_layers(conn: &Connection) -> Result<Vec<Layer>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, name, display_order, created_at FROM layers ORDER BY display_order",
    )?;
    let layers = stmt
        .query_map([], |row| {
            Ok(Layer {
                id: row.get(0)?,
                name: row.get(1)?,
                display_order: row.get(2)?,
                created_at: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
    Ok(layers)
}

pub fn insert_layer(
    conn: &Connection,
    name: &str,
    display_order: i32,
) -> Result<Layer, AppError> {
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    conn.execute(
        "INSERT INTO layers (id, name, display_order, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![id, name, display_order, created_at],
    )?;
    Ok(Layer {
        id,
        name: name.to_string(),
        display_order,
        created_at,
    })
}

// ---------------------------------------------------------------------------
// Edge helpers
// ---------------------------------------------------------------------------

fn query_edges_for_node(
    conn: &Connection,
    node_id: &str,
) -> Result<Vec<EdgeRef>, AppError> {
    let mut stmt = conn.prepare(
        "SELECT id, target_id, edge_type, weight FROM edges WHERE source_id = ?1",
    )?;
    let edges = stmt
        .query_map([node_id], |row| {
            Ok(EdgeRef {
                id: row.get(0)?,
                target_id: row.get(1)?,
                edge_type: row.get(2)?,
                weight: row.get(3)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;
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
        "SELECT id, title, layer_id, node_type, content_type, content_data,
                tags, learned, weight, pos_x, pos_y, pos_z, created_at
         FROM nodes WHERE layer_id = ?1",
    )?;

    let rows: Vec<NodeRow> = stmt
        .query_map([layer_id], |row| {
            Ok(NodeRow {
                id: row.get(0)?,
                title: row.get(1)?,
                layer_id: row.get(2)?,
                node_type: row.get(3)?,
                content_type: row.get(4)?,
                content_data: row.get(5)?,
                tags_json: row.get(6)?,
                learned: row.get::<_, i32>(7)? != 0,
                weight: row.get(8)?,
                pos_x: row.get(9)?,
                pos_y: row.get(10)?,
                pos_z: row.get(11)?,
                created_at: row.get(12)?,
            })
        })?
        .collect::<Result<Vec<_>, _>>()?;

    let mut nodes = Vec::new();
    for row in rows {
        let tags: Vec<String> =
            serde_json::from_str(&row.tags_json).unwrap_or_default();
        let connections = query_edges_for_node(conn, &row.id)?;
        nodes.push(Node {
            id: row.id,
            title: row.title,
            layer_id: row.layer_id,
            node_type: row.node_type,
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
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    let tags_json =
        serde_json::to_string(&input.tags).unwrap_or_else(|_| "[]".to_string());

    conn.execute(
        "INSERT INTO nodes
             (id, title, layer_id, node_type, content_type, content_data, tags, learned, weight, created_at)
         VALUES (?1, ?2, ?3, ?4, 'text', ?5, ?6, 0, ?7, ?8)",
        params![
            id,
            input.title,
            input.layer_id,
            input.node_type,
            input.content_data,
            tags_json,
            input.weight,
            created_at
        ],
    )?;

    Ok(Node {
        id,
        title: input.title,
        layer_id: input.layer_id,
        node_type: input.node_type,
        content_type: "text".to_string(),
        content_data: input.content_data,
        tags: input.tags,
        learned: false,
        weight: input.weight,
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
            "SELECT title, layer_id, node_type, content_type, content_data,
                    tags, learned, weight, pos_x, pos_y, pos_z, created_at
             FROM nodes WHERE id = ?1",
            [id],
            |row| {
                Ok(NodeData {
                    title: row.get(0)?,
                    layer_id: row.get(1)?,
                    node_type: row.get(2)?,
                    content_type: row.get(3)?,
                    content_data: row.get(4)?,
                    tags_json: row.get(5)?,
                    learned: row.get::<_, i32>(6)? != 0,
                    weight: row.get(7)?,
                    pos_x: row.get(8)?,
                    pos_y: row.get(9)?,
                    pos_z: row.get(10)?,
                    created_at: row.get(11)?,
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
    let connections = query_edges_for_node(conn, id)?;

    Ok(Node {
        id: id.to_string(),
        title: data.title,
        layer_id: data.layer_id,
        node_type: data.node_type,
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
    let id = Uuid::new_v4().to_string();
    let created_at = now_ts();
    let weight = 1.0_f64;
    conn.execute(
        "INSERT INTO edges (id, source_id, target_id, edge_type, weight, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, source_id, target_id, edge_type, weight, created_at],
    )?;
    Ok(Edge {
        id,
        source_id: source_id.to_string(),
        target_id: target_id.to_string(),
        edge_type: edge_type.to_string(),
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
// Seed data — Japanese N5 Grammar layer
// ---------------------------------------------------------------------------

pub fn seed_sample_data(conn: &Connection) -> Result<(), AppError> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM layers WHERE name = 'Japanese N5 Grammar'",
        [],
        |row| row.get(0),
    )?;
    if count > 0 {
        return Ok(()); // already seeded
    }

    let layer = insert_layer(conn, "Japanese N5 Grammar", 0)?;

    let nodes_data = [
        ("は (wa)", "grammar", "Topic marker particle — marks the sentence topic"),
        ("が (ga)", "grammar", "Subject marker particle — marks the grammatical subject"),
        ("を (wo)", "grammar", "Object marker particle — marks the direct object"),
        ("ます-form", "grammar", "Polite present/future verb form (e.g. 食べます)"),
        ("て-form", "grammar", "Te-form used to connect actions and form compounds (e.g. 食べて)"),
        ("です (desu)", "grammar", "Polite copula — links subject to predicate"),
    ];

    let mut node_ids: Vec<String> = Vec::new();
    for (title, node_type, content) in &nodes_data {
        let node = insert_node(
            conn,
            CreateNodeInput {
                title: title.to_string(),
                layer_id: layer.id.clone(),
                node_type: node_type.to_string(),
                content_data: Some(content.to_string()),
                tags: vec!["n5".to_string(), "grammar".to_string()],
                weight: 1.0,
            },
        )?;
        node_ids.push(node.id);
    }

    // Indices: は(0) が(1) を(2) ます-form(3) て-form(4) です(5)
    let edges = [
        (0, 1, "Context"),      // は → が
        (0, 2, "Context"),      // は → を
        (3, 4, "Prerequisite"), // ます-form → て-form
        (4, 3, "Context"),      // て-form → ます-form
        (0, 5, "Context"),      // は → です
        (1, 2, "Semantic"),     // が → を
        (3, 5, "Semantic"),     // ます-form → です
        (4, 1, "Context"),      // て-form → が
    ];

    for (src, tgt, edge_type) in &edges {
        insert_edge(conn, &node_ids[*src], &node_ids[*tgt], edge_type)?;
    }

    Ok(())
}
