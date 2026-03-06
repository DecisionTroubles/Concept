use rusqlite::{Connection, Result};

pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "
        PRAGMA journal_mode=WAL;
        PRAGMA foreign_keys=ON;

        CREATE TABLE IF NOT EXISTS layers (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            display_order INTEGER NOT NULL DEFAULT 0,
            filter_json  TEXT NOT NULL DEFAULT '{}',
            metadata     TEXT NOT NULL DEFAULT '{}',
            created_at   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nodes (
            id           TEXT PRIMARY KEY,
            title        TEXT NOT NULL,
            layer_id     TEXT NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            node_type    TEXT NOT NULL DEFAULT 'vocab',
            note_type_id TEXT REFERENCES note_types(id) ON DELETE SET NULL,
            note_fields  TEXT NOT NULL DEFAULT '{}',
            content_type TEXT NOT NULL DEFAULT 'text',
            content_data TEXT,
            tags         TEXT NOT NULL DEFAULT '[]',
            learned      INTEGER NOT NULL DEFAULT 0,
            weight       REAL NOT NULL DEFAULT 1.0,
            pos_x        REAL,
            pos_y        REAL,
            pos_z        REAL,
            metadata     TEXT NOT NULL DEFAULT '{}',
            created_at   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS edges (
            id         TEXT PRIMARY KEY,
            source_id  TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            target_id  TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            edge_type  TEXT NOT NULL DEFAULT 'Context',
            relation_id TEXT REFERENCES relation_kinds(id) ON DELETE SET NULL,
            weight     REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS worlds (
            id          TEXT PRIMARY KEY,
            name        TEXT NOT NULL,
            config_json TEXT NOT NULL DEFAULT '{}',
            created_at  TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS relation_kinds (
            id             TEXT PRIMARY KEY,
            world_id       TEXT NOT NULL REFERENCES worlds(id) ON DELETE CASCADE,
            label          TEXT NOT NULL,
            directed       INTEGER NOT NULL DEFAULT 0,
            default_weight REAL NOT NULL DEFAULT 1.0,
            metadata       TEXT NOT NULL DEFAULT '{}',
            created_at     TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS node_layers (
            node_id     TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            layer_id    TEXT NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            created_at  TEXT NOT NULL,
            PRIMARY KEY (node_id, layer_id)
        );

        CREATE TABLE IF NOT EXISTS edge_layers (
            edge_id     TEXT NOT NULL REFERENCES edges(id) ON DELETE CASCADE,
            layer_id    TEXT NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            created_at  TEXT NOT NULL,
            PRIMARY KEY (edge_id, layer_id)
        );

        CREATE TABLE IF NOT EXISTS connection_layers (
            id           TEXT PRIMARY KEY,
            name         TEXT NOT NULL,
            display_order INTEGER NOT NULL DEFAULT 0,
            metadata     TEXT NOT NULL DEFAULT '{}',
            created_at   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS edge_connection_layers (
            edge_id             TEXT NOT NULL REFERENCES edges(id) ON DELETE CASCADE,
            connection_layer_id TEXT NOT NULL REFERENCES connection_layers(id) ON DELETE CASCADE,
            created_at          TEXT NOT NULL,
            PRIMARY KEY (edge_id, connection_layer_id)
        );

        CREATE TABLE IF NOT EXISTS note_types (
            id         TEXT PRIMARY KEY,
            name       TEXT NOT NULL UNIQUE,
            world_id   TEXT REFERENCES worlds(id) ON DELETE CASCADE,
            base_note_type_id TEXT REFERENCES note_types(id) ON DELETE SET NULL,
            fields     TEXT NOT NULL DEFAULT '[]',
            schema_json TEXT NOT NULL DEFAULT '{}',
            layout_json TEXT NOT NULL DEFAULT '{}',
            metadata   TEXT NOT NULL DEFAULT '{}',
            is_default INTEGER NOT NULL DEFAULT 0,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS node_progress (
            node_id            TEXT PRIMARY KEY REFERENCES nodes(id) ON DELETE CASCADE,
            status             TEXT NOT NULL DEFAULT 'new',
            review_count       INTEGER NOT NULL DEFAULT 0,
            streak             INTEGER NOT NULL DEFAULT 0,
            last_reviewed_at   TEXT,
            next_review_at     TEXT,
            scheduler_key      TEXT NOT NULL DEFAULT 'basic-v1',
            scheduler_state    TEXT NOT NULL DEFAULT '{}',
            created_at         TEXT NOT NULL,
            updated_at         TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS review_events (
            id                 TEXT PRIMARY KEY,
            node_id            TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            grade              TEXT NOT NULL,
            scheduler_key      TEXT NOT NULL,
            reviewed_at        TEXT NOT NULL,
            previous_status    TEXT NOT NULL,
            next_status        TEXT NOT NULL,
            scheduled_for_at   TEXT,
            scheduler_state    TEXT NOT NULL DEFAULT '{}'
        );

        CREATE TABLE IF NOT EXISTS node_extension_data (
            id                 TEXT PRIMARY KEY,
            node_id            TEXT NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
            extension_key      TEXT NOT NULL,
            data_json          TEXT NOT NULL DEFAULT '{}',
            created_at         TEXT NOT NULL,
            updated_at         TEXT NOT NULL,
            UNIQUE(node_id, extension_key)
        );

        CREATE INDEX IF NOT EXISTS idx_nodes_layer   ON nodes(layer_id);
        CREATE INDEX IF NOT EXISTS idx_edges_source  ON edges(source_id);
        CREATE INDEX IF NOT EXISTS idx_edges_target  ON edges(target_id);
        CREATE INDEX IF NOT EXISTS idx_relation_world ON relation_kinds(world_id);
        CREATE INDEX IF NOT EXISTS idx_node_layers_layer ON node_layers(layer_id);
        CREATE INDEX IF NOT EXISTS idx_edge_layers_layer ON edge_layers(layer_id);
        CREATE INDEX IF NOT EXISTS idx_edge_connection_layers_layer ON edge_connection_layers(connection_layer_id);
        CREATE INDEX IF NOT EXISTS idx_node_progress_status ON node_progress(status);
        CREATE INDEX IF NOT EXISTS idx_node_progress_next_review ON node_progress(next_review_at);
        CREATE INDEX IF NOT EXISTS idx_review_events_node_id ON review_events(node_id);
        CREATE INDEX IF NOT EXISTS idx_review_events_reviewed_at ON review_events(reviewed_at);
        CREATE INDEX IF NOT EXISTS idx_node_extension_data_node ON node_extension_data(node_id);
        CREATE INDEX IF NOT EXISTS idx_node_extension_data_extension_key ON node_extension_data(extension_key);
        ",
    )?;

    // Forward-compatible migration for pre-note-type databases.
    let mut stmt = conn.prepare("PRAGMA table_info(nodes)")?;
    let cols = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;

    if !cols.iter().any(|c| c == "note_type_id") {
        conn.execute("ALTER TABLE nodes ADD COLUMN note_type_id TEXT REFERENCES note_types(id) ON DELETE SET NULL", [])?;
    }
    if !cols.iter().any(|c| c == "note_fields") {
        conn.execute(
            "ALTER TABLE nodes ADD COLUMN note_fields TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }

    let mut layer_stmt = conn.prepare("PRAGMA table_info(layers)")?;
    let layer_cols = layer_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !layer_cols.iter().any(|c| c == "filter_json") {
        conn.execute(
            "ALTER TABLE layers ADD COLUMN filter_json TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }
    if !layer_cols.iter().any(|c| c == "metadata") {
        conn.execute(
            "ALTER TABLE layers ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }

    let mut edge_stmt = conn.prepare("PRAGMA table_info(edges)")?;
    let edge_cols = edge_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !edge_cols.iter().any(|c| c == "relation_id") {
        conn.execute("ALTER TABLE edges ADD COLUMN relation_id TEXT REFERENCES relation_kinds(id) ON DELETE SET NULL", [])?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_edges_relation_id ON edges(relation_id)",
        [],
    )?;

    // Create note-type index only after migration columns are guaranteed to exist.
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_nodes_note_type ON nodes(note_type_id)",
        [],
    )?;

    let mut note_type_stmt = conn.prepare("PRAGMA table_info(note_types)")?;
    let note_type_cols = note_type_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !note_type_cols.iter().any(|c| c == "schema_json") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN schema_json TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }
    if !note_type_cols.iter().any(|c| c == "layout_json") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN layout_json TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }
    if !note_type_cols.iter().any(|c| c == "world_id") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN world_id TEXT REFERENCES worlds(id) ON DELETE CASCADE",
            [],
        )?;
    }
    if !note_type_cols.iter().any(|c| c == "base_note_type_id") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN base_note_type_id TEXT REFERENCES note_types(id) ON DELETE SET NULL",
            [],
        )?;
    }
    if !note_type_cols.iter().any(|c| c == "metadata") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN metadata TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }
    if !note_type_cols.iter().any(|c| c == "updated_at") {
        conn.execute(
            "ALTER TABLE note_types ADD COLUMN updated_at TEXT NOT NULL DEFAULT '0'",
            [],
        )?;
        conn.execute(
            "UPDATE note_types SET updated_at = created_at WHERE updated_at = '0'",
            [],
        )?;
    }

    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_note_types_world_id ON note_types(world_id)",
        [],
    )?;
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_note_types_base_note_type ON note_types(base_note_type_id)",
        [],
    )?;

    let mut progress_stmt = conn.prepare("PRAGMA table_info(node_progress)")?;
    let progress_cols = progress_stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<Result<Vec<_>, _>>()?;
    if !progress_cols.iter().any(|c| c == "scheduler_key") {
        conn.execute(
            "ALTER TABLE node_progress ADD COLUMN scheduler_key TEXT NOT NULL DEFAULT 'basic-v1'",
            [],
        )?;
    }
    if !progress_cols.iter().any(|c| c == "scheduler_state") {
        conn.execute(
            "ALTER TABLE node_progress ADD COLUMN scheduler_state TEXT NOT NULL DEFAULT '{}'",
            [],
        )?;
    }

    // Run one-time migration to reconcile duplicate layers from v1→v2 transition
    reconcile_duplicate_layers(conn)?;

    // Run one-time migration to separate connection layers from node visibility layers
    migrate_edge_layers_to_connection_layers(conn)?;

    Ok(())
}

/// One-time migration: Reconcile duplicate layer names from v1→v2 transition.
///
/// Background:
/// - Old v1 DBs created layers by name (e.g., "Grammar" had some UUID)
/// - New v2 loader calls insert_layer() which generates a fresh UUID
/// - First v2 seed run: old "Grammar" + new "Grammar" → duplicates
///
/// Solution:
/// - Find all layers grouped by name
/// - For each group with duplicates, keep the oldest (by created_at)
/// - Remap all node_layers and edge_layers references to the keeper layer
/// - Delete duplicate layers (cascades to orphaned memberships)
fn reconcile_duplicate_layers(conn: &Connection) -> Result<()> {
    // Find all layers grouped by name with duplicates
    let mut stmt =
        conn.prepare("SELECT name, COUNT(*) as cnt FROM layers GROUP BY name HAVING cnt > 1")?;
    let duplicate_names: Vec<String> = stmt
        .query_map([], |row| row.get(0))?
        .collect::<Result<Vec<_>, _>>()?;

    for name in duplicate_names {
        // Get all layers with this name, sorted by created_at (oldest first)
        let mut layer_stmt =
            conn.prepare("SELECT id FROM layers WHERE name = ?1 ORDER BY created_at ASC")?;
        let layer_ids: Vec<String> = layer_stmt
            .query_map([&name], |row| row.get(0))?
            .collect::<Result<Vec<_>, _>>()?;

        if layer_ids.is_empty() {
            continue;
        }

        let keeper_id = &layer_ids[0]; // Keep the oldest layer
        let duplicate_ids = &layer_ids[1..]; // Delete the rest

        for dup_id in duplicate_ids {
            // Remap node_layers: old duplicate_id → keeper_id
            conn.execute(
                "UPDATE node_layers SET layer_id = ?1 WHERE layer_id = ?2",
                rusqlite::params![keeper_id, dup_id],
            )?;

            // Remap edge_layers: old duplicate_id → keeper_id
            conn.execute(
                "UPDATE edge_layers SET layer_id = ?1 WHERE layer_id = ?2",
                rusqlite::params![keeper_id, dup_id],
            )?;

            // Update nodes that reference the duplicate layer
            conn.execute(
                "UPDATE nodes SET layer_id = ?1 WHERE layer_id = ?2",
                rusqlite::params![keeper_id, dup_id],
            )?;

            // Delete the duplicate layer (cascade deletes orphaned memberships if any)
            conn.execute(
                "DELETE FROM layers WHERE id = ?1",
                rusqlite::params![dup_id],
            )?;
        }
    }

    Ok(())
}

/// One-time migration: Migrate edge_layers to edge_connection_layers with connection layer wrapping.
///
/// Background:
/// - Originally, edge_layers controlled both node visibility AND edge overlay membership
/// - New design separates concerns:
///   * layers + node_layers: controls which nodes are visible in a layer
///   * connection_layers + edge_connection_layers: controls which edge overlays are active
/// - This migration preserves existing edge layer memberships by creating corresponding connection layers
///
/// Migration logic:
/// 1. Check if connection_layers table is empty (indicates first run)
/// 2. If edge_layers has data and connection_layers is empty, create a default connection layer
/// 3. Copy all edge_layers entries to edge_connection_layers (remapped to the connection layer)
/// 4. Keep edge_layers for backward compatibility
fn migrate_edge_layers_to_connection_layers(conn: &Connection) -> Result<()> {
    use std::time::SystemTime;
    use uuid::Uuid;

    // Check if migration has already run
    let mut check_stmt = conn.prepare("SELECT COUNT(*) FROM connection_layers")?;
    let connection_layer_count: i64 = check_stmt.query_row([], |row| row.get(0))?;

    if connection_layer_count > 0 {
        // Migration already completed
        return Ok(());
    }

    // Check if there's any data in edge_layers to migrate
    let mut edge_check_stmt = conn.prepare("SELECT COUNT(*) FROM edge_layers")?;
    let edge_layer_count: i64 = edge_check_stmt.query_row([], |row| row.get(0))?;

    if edge_layer_count == 0 {
        // No data to migrate, but create a default connection layer for future use
        let default_id = Uuid::new_v4().to_string();
        let now_ts = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .map(|d| d.as_secs().to_string())
            .unwrap_or_else(|_| "0".to_string());

        conn.execute(
            "INSERT INTO connection_layers (id, name, display_order, metadata, created_at)
             VALUES (?1, ?2, ?3, '{}', ?4)",
            rusqlite::params![default_id, "Default", 0, now_ts],
        )?;
        return Ok(());
    }

    // Get or create a single connection layer to hold all migrated edges
    // Use the first layer name from edge_layers as the connection layer name
    let mut name_stmt = conn.prepare(
        "SELECT DISTINCT l.name FROM edge_layers el
         JOIN layers l ON el.layer_id = l.id
         ORDER BY l.display_order LIMIT 1",
    )?;
    let connection_layer_name = name_stmt
        .query_row([], |row| row.get::<_, String>(0))
        .unwrap_or_else(|_| "Edge Overlays".to_string());

    let connection_layer_id = Uuid::new_v4().to_string();
    let now_ts = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string());

    conn.execute(
        "INSERT INTO connection_layers (id, name, display_order, metadata, created_at)
         VALUES (?1, ?2, ?3, '{}', ?4)",
        rusqlite::params![connection_layer_id, connection_layer_name, 0, now_ts],
    )?;

    // Migrate all edge_layers to edge_connection_layers
    // Each edge that was in any layer becomes a member of this connection layer
    let mut migrate_stmt = conn.prepare("SELECT edge_id, created_at FROM edge_layers")?;
    let edge_records: Vec<(String, String)> = migrate_stmt
        .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
        .collect::<Result<Vec<_>, _>>()?;

    for (edge_id, created_at) in edge_records {
        conn.execute(
            "INSERT OR IGNORE INTO edge_connection_layers (edge_id, connection_layer_id, created_at)
             VALUES (?1, ?2, ?3)",
            rusqlite::params![edge_id, connection_layer_id, created_at],
        )?;
    }

    Ok(())
}
