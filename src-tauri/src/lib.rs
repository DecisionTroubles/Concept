mod db;
mod domain;
mod error;
mod graph;

use std::sync::{Arc, OnceLock};

use rusqlite::Connection;
use tauri::Manager;
use tokio::sync::Mutex;

use graph::{CreateNodeInput, Edge, Layer, Node};

// ---------------------------------------------------------------------------
// DB state — initialized once in setup, shared across all resolver calls.
// OnceLock is safe as a static because Arc<Mutex<Connection>> is Send + Sync.
// ---------------------------------------------------------------------------

type DbState = Arc<Mutex<Connection>>;

static DB: OnceLock<DbState> = OnceLock::new();

fn db() -> &'static DbState {
    DB.get().expect("DB not initialized — setup() has not run yet")
}

// ---------------------------------------------------------------------------
// TauRPC API definition — generates src/bindings.ts
// ---------------------------------------------------------------------------

#[taurpc::procedures(export_to = "../src/bindings.ts")]
trait GraphApi {
    // Layers
    async fn get_layers() -> Result<Vec<Layer>, String>;
    async fn create_layer(name: String, display_order: i32) -> Result<Layer, String>;

    // Nodes — edges embedded, no second IPC round-trip
    async fn get_nodes(layer_id: String) -> Result<Vec<Node>, String>;
    async fn create_node(input: CreateNodeInput) -> Result<Node, String>;
    async fn mark_learned(id: String, learned: bool) -> Result<Node, String>;
    async fn update_node_position(id: String, x: f64, y: f64, z: f64) -> Result<(), String>;

    // Edges
    async fn create_edge(
        source_id: String,
        target_id: String,
        edge_type: String,
    ) -> Result<Edge, String>;
    async fn delete_edge(id: String) -> Result<(), String>;

    // Dev / seed
    async fn seed_sample_data() -> Result<(), String>;
    async fn reset_data() -> Result<(), String>;
}

// ---------------------------------------------------------------------------
// Resolver implementation
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct ApiImpl;

#[taurpc::resolvers]
impl GraphApi for ApiImpl {
    async fn get_layers(self) -> Result<Vec<Layer>, String> {
        let conn = db().lock().await;
        graph::query_layers(&conn).map_err(|e| e.to_string())
    }

    async fn create_layer(self, name: String, display_order: i32) -> Result<Layer, String> {
        let conn = db().lock().await;
        graph::insert_layer(&conn, &name, display_order).map_err(|e| e.to_string())
    }

    async fn get_nodes(self, layer_id: String) -> Result<Vec<Node>, String> {
        let conn = db().lock().await;
        graph::query_nodes(&conn, &layer_id).map_err(|e| e.to_string())
    }

    async fn create_node(self, input: CreateNodeInput) -> Result<Node, String> {
        let conn = db().lock().await;
        graph::insert_node(&conn, input).map_err(|e| e.to_string())
    }

    async fn mark_learned(self, id: String, learned: bool) -> Result<Node, String> {
        let conn = db().lock().await;
        graph::set_learned(&conn, &id, learned).map_err(|e| e.to_string())
    }

    async fn update_node_position(self, id: String, x: f64, y: f64, z: f64) -> Result<(), String> {
        let conn = db().lock().await;
        graph::set_node_position(&conn, &id, x, y, z).map_err(|e| e.to_string())
    }

    async fn create_edge(
        self,
        source_id: String,
        target_id: String,
        edge_type: String,
    ) -> Result<Edge, String> {
        let conn = db().lock().await;
        graph::insert_edge(&conn, &source_id, &target_id, &edge_type)
            .map_err(|e| e.to_string())
    }

    async fn delete_edge(self, id: String) -> Result<(), String> {
        let conn = db().lock().await;
        graph::remove_edge(&conn, &id).map_err(|e| e.to_string())
    }

    async fn seed_sample_data(self) -> Result<(), String> {
        let conn = db().lock().await;
        graph::seed_sample_data(&conn).map_err(|e| e.to_string())
    }

    async fn reset_data(self) -> Result<(), String> {
        let conn = db().lock().await;
        graph::reset_and_reseed(&conn).map_err(|e| e.to_string())
    }
}

// ---------------------------------------------------------------------------
// App entry point
// ---------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub async fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(debug_assertions)]
            {
                let window = app.get_webview_window("main").unwrap();
                window.open_devtools();
            }

            // Open / create the SQLite database in the app data directory.
            let data_dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&data_dir)?;
            let conn = Connection::open(data_dir.join("graph.db"))?;
            db::init_schema(&conn)?;
            // Store in the global OnceLock — safe because setup() runs exactly once
            // before any IPC invocations can arrive.
            DB.set(Arc::new(Mutex::new(conn)))
                .map_err(|_| "DB already initialized")?;

            Ok(())
        })
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_prevent_default::init())
        .invoke_handler(taurpc::create_ipc_handler(ApiImpl.into_handler()))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
