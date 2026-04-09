use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::Deserialize;
use toml::Value as TomlValue;

#[derive(Clone, Debug)]
pub struct SourcePack {
    pub root_dir: PathBuf,
    pub pack_file: PathBuf,
    pub manifest: SourcePackManifest,
    pub theme: Option<SourceTheme>,
    pub groups: Vec<SourceGroup>,
    pub layers: Vec<SourceLayer>,
    pub connection_layers: Vec<SourceConnectionLayer>,
    pub relation_kinds: Vec<SourceRelationKind>,
    pub note_types: Vec<SourceNoteType>,
    pub nodes: Vec<SourceNode>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourcePackManifest {
    pub version: String,
    pub world: SourceWorld,
    #[serde(default)]
    pub authoring: SourceAuthoring,
    #[serde(default)]
    pub layout: SourceLayoutHints,
    #[serde(default)]
    pub build: SourceBuild,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceWorld {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub root_node: Option<String>,
    #[serde(default)]
    pub default_note_type: Option<String>,
    #[serde(default)]
    pub metadata: Option<TomlValue>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourceAuthoring {
    #[serde(default)]
    pub default_group: Option<String>,
    #[serde(default)]
    pub default_layer: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourceLayoutHints {
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub node_spacing: Option<f64>,
    #[serde(default)]
    pub group_spacing: Option<f64>,
    #[serde(default)]
    pub focus_child_radius: Option<f64>,
    #[serde(default)]
    pub allow_explicit_positions: Option<bool>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourceBuild {
    #[serde(default)]
    pub emit_runtime_pack: Option<bool>,
    #[serde(default)]
    pub runtime_pack_version: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourceTheme {
    #[serde(default)]
    pub node_types: BTreeMap<String, SourceThemeNodeTypeStyle>,
    #[serde(default)]
    pub labels: Option<TomlValue>,
    #[serde(default)]
    pub focus: Option<TomlValue>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourceThemeNodeTypeStyle {
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub emissive: Option<String>,
    #[serde(default)]
    pub radius: Option<f64>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceGroup {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub style: Option<TomlValue>,
    #[serde(default)]
    pub layout: Option<TomlValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceLayer {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub display_order: Option<i32>,
    #[serde(default)]
    pub style: Option<TomlValue>,
    #[serde(default)]
    pub layout: Option<TomlValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceConnectionLayer {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub display_order: Option<i32>,
    #[serde(default)]
    pub style: Option<TomlValue>,
    #[serde(default)]
    pub layout: Option<TomlValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceRelationKind {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub directed: Option<bool>,
    #[serde(default)]
    pub default_weight: Option<f64>,
    #[serde(default)]
    pub style: Option<TomlValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceNoteType {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub base_note_type_id: Option<String>,
    #[serde(default)]
    pub is_default: Option<bool>,
    #[serde(default)]
    pub fields: Vec<SourceNoteTypeField>,
    #[serde(default)]
    pub pages: Vec<SourceNoteTypePage>,
    #[serde(default)]
    pub metadata: Option<TomlValue>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceNoteTypeField {
    pub key: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default, rename = "type")]
    pub field_type: Option<String>,
    #[serde(default)]
    pub widget: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceNoteTypePage {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub fields: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct SourceNode {
    pub file_path: PathBuf,
    pub frontmatter: SourceNodeFrontmatter,
    pub body: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceNodeFrontmatter {
    pub id: String,
    pub title: String,
    pub node_type: String,
    pub note_type: String,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub layer: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parent: Option<String>,
    #[serde(default)]
    pub placement: Option<SourcePlacement>,
    #[serde(default)]
    pub links: Vec<SourceNodeLink>,
    #[serde(default)]
    pub style_override: Option<TomlValue>,
    #[serde(default)]
    pub metadata: Option<TomlValue>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SourcePlacement {
    #[serde(default)]
    pub x: Option<f64>,
    #[serde(default)]
    pub y: Option<f64>,
    #[serde(default)]
    pub z: Option<f64>,
    #[serde(default)]
    pub locked: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SourceNodeLink {
    pub to: String,
    pub relation: String,
    pub layers: Vec<String>,
    #[serde(default)]
    pub weight: Option<f64>,
    #[serde(default)]
    pub bidirectional: Option<bool>,
    #[serde(default)]
    pub metadata: Option<TomlValue>,
}
