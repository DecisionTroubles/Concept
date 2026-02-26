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
            created_at   TEXT NOT NULL
        );

        CREATE TABLE IF NOT EXISTS nodes (
            id           TEXT PRIMARY KEY,
            title        TEXT NOT NULL,
            layer_id     TEXT NOT NULL REFERENCES layers(id) ON DELETE CASCADE,
            node_type    TEXT NOT NULL DEFAULT 'vocab',
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
            weight     REAL NOT NULL DEFAULT 1.0,
            created_at TEXT NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_nodes_layer   ON nodes(layer_id);
        CREATE INDEX IF NOT EXISTS idx_edges_source  ON edges(source_id);
        CREATE INDEX IF NOT EXISTS idx_edges_target  ON edges(target_id);
        ",
    )
}
