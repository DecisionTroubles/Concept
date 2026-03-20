mod db;
mod domain;
mod error;
mod extensions;
mod graph;
mod pack_registry;
mod scheduler;
mod world_registry;

use std::fs;
use std::path::Path;
use std::sync::{Arc, OnceLock};

use rusqlite::Connection;
use tauri::Manager;
use tokio::sync::Mutex;

use graph::{
    ConnectionLayer, CreateNodeInput, Edge, Layer, Node, NodeProgress, NoteType, NoteTypeInput, RelationKind, WorldConfig,
};
use extensions::NodeExtensionData;
use scheduler::{ReviewEvent, SchedulerDescriptor};
use world_registry::{ScanRoot, WorldPackInfo};
use pack_registry::{GitHubPackSourceInput, PackRegistryEntry};

// ---------------------------------------------------------------------------
// DB state — initialized once in setup, shared across all resolver calls.
// OnceLock is safe as a static because Arc<Mutex<Connection>> is Send + Sync.
// ---------------------------------------------------------------------------

type DbState = Arc<Mutex<Connection>>;

static DB: OnceLock<DbState> = OnceLock::new();

fn db() -> &'static DbState {
    DB.get()
        .expect("DB not initialized — setup() has not run yet")
}

fn ensure_starter_pack(local_worlds_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let starter_dir = local_worlds_dir.join("starter-example");
    let starter_pack = starter_dir.join("pack.json");
    if starter_pack.exists() {
        return Ok(());
    }

    fs::create_dir_all(&starter_dir)?;
    fs::write(
        &starter_pack,
        r#"{
  "version": "2",
  "world": {
    "id": "starter-example",
    "name": "Starter Example",
    "layout": {},
    "metadata": {
      "description": "A tiny example world shipped into app data as a local starter pack."
    }
  },
  "note_types": [
    {
      "id": "starter-concept",
      "name": "Starter Concept",
      "fields": ["Summary", "Why", "Example", "Pitfall"],
      "schema_json": {
        "fields": [
          { "key": "Summary", "label": "Summary", "widget": "text" },
          { "key": "Why", "label": "Why", "widget": "textarea" },
          { "key": "Example", "label": "Example", "widget": "code" },
          { "key": "Pitfall", "label": "Pitfall", "widget": "textarea" }
        ]
      },
      "layout_json": {},
      "metadata": {},
      "is_default": true
    }
  ],
  "relation_kinds": [
    { "id": "rel-explains", "label": "Explains", "directed": true, "default_weight": 1.0, "metadata": {} },
    { "id": "rel-next", "label": "Next", "directed": true, "default_weight": 1.0, "metadata": {} }
  ],
  "layers": [
    { "id": "main", "name": "Main", "display_order": 0, "node_filter": {}, "edge_filter": {}, "metadata": {} }
  ],
  "connection_layers": [
    { "id": "all-links", "name": "All links", "display_order": 0, "metadata": {} }
  ],
  "nodes": [
    {
      "id": "welcome",
      "title": "Welcome",
      "node_type": "concept",
      "note_type_id": "starter-concept",
      "note_fields": {
        "Summary": "This starter world shows the basic reading flow of Concept.",
        "Why": "Use it as a small sanity-check world before installing larger packs from GitHub.",
        "Example": "Open Welcome, move to Connections, then inspect Graph basics.",
        "Pitfall": "Treating the starter pack as real content instead of a template will limit the map."
      },
      "content_data": "Welcome to Concept.",
      "tags": ["starter"],
      "weight": 1.0,
      "position": { "x": 0.0, "y": 0.0, "z": 0.0 },
      "layer_membership": ["main"],
      "metadata": {}
    },
    {
      "id": "graph-basics",
      "title": "Graph basics",
      "node_type": "concept",
      "note_type_id": "starter-concept",
      "note_fields": {
        "Summary": "Nodes carry ideas, and links tell you how to move through them.",
        "Why": "A small graph teaches the viewer and navigation model without content overload.",
        "Example": "Welcome -> Graph basics -> Install packs",
        "Pitfall": "Raw links without explanation make a map feel mechanical."
      },
      "content_data": "Graph basics.",
      "tags": ["starter"],
      "weight": 1.0,
      "position": { "x": 4.0, "y": 0.0, "z": -1.5 },
      "layer_membership": ["main"],
      "metadata": {}
    },
    {
      "id": "install-packs",
      "title": "Install packs",
      "node_type": "concept",
      "note_type_id": "starter-concept",
      "note_fields": {
        "Summary": "Open the pack library, add GitHub sources there, then pull them into your local pack library.",
        "Why": "Runtime packs now come from app data, not bundled domains in the repo.",
        "Example": "Projects -> Pack library -> Add source -> Pull -> Open project",
        "Pitfall": "Expecting repo domains to appear at runtime will not work anymore."
      },
      "content_data": "Install packs from GitHub.",
      "tags": ["starter"],
      "weight": 1.0,
      "position": { "x": 8.0, "y": 0.0, "z": 1.5 },
      "layer_membership": ["main"],
      "metadata": {}
    }
  ],
  "edges": [
    {
      "id": "edge-welcome-graph",
      "source_id": "welcome",
      "target_id": "graph-basics",
      "relation_id": "rel-explains",
      "edge_type": "Semantic",
      "weight": 1.0,
      "connection_layer_membership": ["all-links"],
      "metadata": {}
    },
    {
      "id": "edge-graph-install",
      "source_id": "graph-basics",
      "target_id": "install-packs",
      "relation_id": "rel-next",
      "edge_type": "Context",
      "weight": 1.0,
      "connection_layer_membership": ["all-links"],
      "metadata": {}
    }
  ]
}"#,
    )?;

    Ok(())
}

// ---------------------------------------------------------------------------
// TauRPC API definition — generates src/bindings.ts
// ---------------------------------------------------------------------------

#[taurpc::procedures(export_to = "../src/bindings.ts")]
trait GraphApi {
    // Layers
    async fn get_layers() -> Result<Vec<Layer>, String>;
    async fn get_world_config() -> Result<Option<WorldConfig>, String>;
    async fn get_world_packs() -> Result<Vec<WorldPackInfo>, String>;
    async fn get_pack_registry() -> Result<Vec<PackRegistryEntry>, String>;
    async fn get_relation_kinds() -> Result<Vec<RelationKind>, String>;
    async fn get_connection_layers() -> Result<Vec<ConnectionLayer>, String>;
    async fn create_layer(name: String, display_order: i32) -> Result<Layer, String>;

    // Nodes — edges embedded, no second IPC round-trip
    async fn get_nodes(layer_id: String) -> Result<Vec<Node>, String>;
    async fn create_node(input: CreateNodeInput) -> Result<Node, String>;
    async fn mark_learned(id: String, learned: bool) -> Result<Node, String>;
    async fn update_node_position(id: String, x: f64, y: f64, z: f64) -> Result<(), String>;
    async fn get_note_types() -> Result<Vec<NoteType>, String>;
    async fn get_note_type(id: String) -> Result<NoteType, String>;
    async fn get_node_progress() -> Result<Vec<NodeProgress>, String>;
    async fn get_scheduler_algorithms() -> Result<Vec<SchedulerDescriptor>, String>;
    async fn get_review_events() -> Result<Vec<ReviewEvent>, String>;
    async fn get_node_extension_data(
        node_id: String,
        extension_key: Option<String>,
    ) -> Result<Vec<NodeExtensionData>, String>;
    async fn set_node_extension_data(
        node_id: String,
        extension_key: String,
        data_json: String,
    ) -> Result<NodeExtensionData, String>;
    async fn create_note_type(input: NoteTypeInput) -> Result<NoteType, String>;
    async fn update_note_type(id: String, input: NoteTypeInput) -> Result<NoteType, String>;
    async fn duplicate_note_type(
        source_id: String,
        name: String,
        world_id: Option<String>,
    ) -> Result<NoteType, String>;
    async fn set_node_note_type(
        node_id: String,
        note_type_id: Option<String>,
    ) -> Result<Node, String>;
    async fn update_node_content(
        node_id: String,
        title: String,
        note_fields: std::collections::BTreeMap<String, String>,
        content_data: Option<String>,
        tags: Vec<String>,
    ) -> Result<Node, String>;
    async fn set_node_progress_status(node_id: String, status: String) -> Result<Node, String>;
    async fn review_node(
        node_id: String,
        grade: String,
        scheduler_key: Option<String>,
    ) -> Result<Node, String>;

    // Edges
    async fn create_edge(
        source_id: String,
        target_id: String,
        edge_type: String,
    ) -> Result<Edge, String>;
    async fn delete_edge(id: String) -> Result<(), String>;

    // Dev / seed
    async fn seed_sample_data() -> Result<(), String>;
    async fn reset_data(reseed: Option<bool>) -> Result<(), String>;
    async fn select_world(world_id: String) -> Result<(), String>;
    async fn reload_active_world() -> Result<(), String>;
    async fn add_github_pack_source(input: GitHubPackSourceInput) -> Result<PackRegistryEntry, String>;
    async fn update_pack_source(id: String, input: GitHubPackSourceInput) -> Result<PackRegistryEntry, String>;
    async fn remove_pack_source(id: String) -> Result<(), String>;
    async fn install_pack_source(id: String) -> Result<PackRegistryEntry, String>;
    async fn refresh_pack_source(id: String) -> Result<PackRegistryEntry, String>;
    async fn check_pack_source_updates(id: String) -> Result<PackRegistryEntry, String>;
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

    async fn get_world_config(self) -> Result<Option<WorldConfig>, String> {
        let conn = db().lock().await;
        graph::query_world_config(&conn).map_err(|e| e.to_string())
    }

    async fn get_world_packs(self) -> Result<Vec<WorldPackInfo>, String> {
        let conn = db().lock().await;
        world_registry::list_world_packs(&conn).map_err(|e| e.to_string())
    }

    async fn get_pack_registry(self) -> Result<Vec<PackRegistryEntry>, String> {
        pack_registry::get_pack_registry().map_err(|e| e.to_string())
    }

    async fn get_relation_kinds(self) -> Result<Vec<RelationKind>, String> {
        let conn = db().lock().await;
        graph::query_relation_kinds(&conn).map_err(|e| e.to_string())
    }

    async fn get_connection_layers(self) -> Result<Vec<ConnectionLayer>, String> {
        let conn = db().lock().await;
        graph::query_connection_layers(&conn).map_err(|e| e.to_string())
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

    async fn get_note_types(self) -> Result<Vec<NoteType>, String> {
        let conn = db().lock().await;
        graph::query_note_types(&conn).map_err(|e| e.to_string())
    }

    async fn get_note_type(self, id: String) -> Result<NoteType, String> {
        let conn = db().lock().await;
        graph::query_note_type(&conn, &id).map_err(|e| e.to_string())
    }

    async fn get_node_progress(self) -> Result<Vec<NodeProgress>, String> {
        let conn = db().lock().await;
        graph::query_node_progress(&conn).map_err(|e| e.to_string())
    }

    async fn get_scheduler_algorithms(self) -> Result<Vec<SchedulerDescriptor>, String> {
        Ok(scheduler::query_scheduler_descriptors())
    }

    async fn get_review_events(self) -> Result<Vec<ReviewEvent>, String> {
        let conn = db().lock().await;
        scheduler::query_review_events(&conn).map_err(|e| e.to_string())
    }

    async fn get_node_extension_data(
        self,
        node_id: String,
        extension_key: Option<String>,
    ) -> Result<Vec<NodeExtensionData>, String> {
        let conn = db().lock().await;
        extensions::query_node_extension_data(&conn, &node_id, extension_key.as_deref()).map_err(|e| e.to_string())
    }

    async fn set_node_extension_data(
        self,
        node_id: String,
        extension_key: String,
        data_json: String,
    ) -> Result<NodeExtensionData, String> {
        let conn = db().lock().await;
        extensions::upsert_node_extension_data(&conn, &node_id, &extension_key, &data_json).map_err(|e| e.to_string())
    }

    async fn create_note_type(self, input: NoteTypeInput) -> Result<NoteType, String> {
        let conn = db().lock().await;
        graph::insert_note_type(&conn, input).map_err(|e| e.to_string())
    }

    async fn update_note_type(self, id: String, input: NoteTypeInput) -> Result<NoteType, String> {
        let conn = db().lock().await;
        graph::update_note_type(&conn, &id, input).map_err(|e| e.to_string())
    }

    async fn duplicate_note_type(
        self,
        source_id: String,
        name: String,
        world_id: Option<String>,
    ) -> Result<NoteType, String> {
        let conn = db().lock().await;
        graph::duplicate_note_type(&conn, &source_id, &name, world_id).map_err(|e| e.to_string())
    }

    async fn set_node_note_type(
        self,
        node_id: String,
        note_type_id: Option<String>,
    ) -> Result<Node, String> {
        let conn = db().lock().await;
        graph::set_node_note_type(&conn, &node_id, note_type_id).map_err(|e| e.to_string())
    }

    async fn update_node_content(
        self,
        node_id: String,
        title: String,
        note_fields: std::collections::BTreeMap<String, String>,
        content_data: Option<String>,
        tags: Vec<String>,
    ) -> Result<Node, String> {
        let conn = db().lock().await;
        graph::update_node_content(&conn, &node_id, title, note_fields, content_data, tags).map_err(|e| e.to_string())
    }

    async fn set_node_progress_status(self, node_id: String, status: String) -> Result<Node, String> {
        let conn = db().lock().await;
        graph::set_node_progress_status(&conn, &node_id, &status).map_err(|e| e.to_string())
    }

    async fn review_node(
        self,
        node_id: String,
        grade: String,
        scheduler_key: Option<String>,
    ) -> Result<Node, String> {
        let conn = db().lock().await;
        scheduler::review_node(&conn, &node_id, &grade, scheduler_key.as_deref()).map_err(|e| e.to_string())
    }

    async fn create_edge(
        self,
        source_id: String,
        target_id: String,
        edge_type: String,
    ) -> Result<Edge, String> {
        let conn = db().lock().await;
        graph::insert_edge(&conn, &source_id, &target_id, &edge_type).map_err(|e| e.to_string())
    }

    async fn delete_edge(self, id: String) -> Result<(), String> {
        let conn = db().lock().await;
        graph::remove_edge(&conn, &id).map_err(|e| e.to_string())
    }

    async fn seed_sample_data(self) -> Result<(), String> {
        let conn = db().lock().await;
        graph::seed_sample_data(&conn).map_err(|e| e.to_string())
    }

    async fn reset_data(self, reseed: Option<bool>) -> Result<(), String> {
        let conn = db().lock().await;
        graph::reset_data(&conn, reseed.unwrap_or(true)).map_err(|e| e.to_string())
    }

    async fn select_world(self, world_id: String) -> Result<(), String> {
        let conn = db().lock().await;
        world_registry::select_world(&conn, &world_id).map_err(|e| e.to_string())
    }

    async fn reload_active_world(self) -> Result<(), String> {
        let conn = db().lock().await;
        world_registry::reload_active_world(&conn).map_err(|e| e.to_string())
    }

    async fn add_github_pack_source(self, input: GitHubPackSourceInput) -> Result<PackRegistryEntry, String> {
        pack_registry::add_github_pack_source(input).map_err(|e| e.to_string())
    }

    async fn update_pack_source(self, id: String, input: GitHubPackSourceInput) -> Result<PackRegistryEntry, String> {
        pack_registry::update_pack_source(&id, input).map_err(|e| e.to_string())
    }

    async fn remove_pack_source(self, id: String) -> Result<(), String> {
        pack_registry::remove_pack_source(&id).map_err(|e| e.to_string())
    }

    async fn install_pack_source(self, id: String) -> Result<PackRegistryEntry, String> {
        pack_registry::install_pack_source(&id).await.map_err(|e| e.to_string())
    }

    async fn refresh_pack_source(self, id: String) -> Result<PackRegistryEntry, String> {
        pack_registry::refresh_pack_source(&id).await.map_err(|e| e.to_string())
    }

    async fn check_pack_source_updates(self, id: String) -> Result<PackRegistryEntry, String> {
        pack_registry::check_pack_source_updates(&id).await.map_err(|e| e.to_string())
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
            let user_worlds_dir = data_dir.join("worlds");
            std::fs::create_dir_all(&user_worlds_dir)?;
            let installed_worlds_dir = user_worlds_dir.join("installed");
            let local_worlds_dir = user_worlds_dir.join("local");
            std::fs::create_dir_all(&installed_worlds_dir)?;
            std::fs::create_dir_all(&local_worlds_dir)?;
            ensure_starter_pack(&local_worlds_dir)?;
            let conn = Connection::open(data_dir.join("graph.db"))?;
            db::init_schema(&conn)?;
            let mut scan_roots = Vec::new();

            scan_roots.push(ScanRoot {
                kind: "installed".into(),
                path: installed_worlds_dir.clone(),
            });

            scan_roots.push(ScanRoot {
                kind: "local".into(),
                path: local_worlds_dir.clone(),
            });

            pack_registry::configure(user_worlds_dir.clone())?;
            world_registry::configure_scan_roots(scan_roots)?;
            world_registry::ensure_active_world_loaded(&conn)?;
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
