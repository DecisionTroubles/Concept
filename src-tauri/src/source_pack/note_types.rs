use serde_json::json;

use crate::source_pack::toml_value_to_json;
use crate::source_pack::types::{SourceNoteType, SourceNoteTypeField, SourceNoteTypePage};

fn field_widget(field: &SourceNoteTypeField) -> String {
    field
        .widget
        .clone()
        .or_else(|| field.field_type.clone())
        .unwrap_or_else(|| "text".into())
}

pub fn runtime_fields(note_type: &SourceNoteType) -> Vec<String> {
    note_type.fields.iter().map(|field| field.key.clone()).collect()
}

pub fn runtime_schema_json(note_type: &SourceNoteType) -> serde_json::Value {
    json!({
        "version": 1,
        "fields": note_type.fields.iter().map(|field| {
            json!({
                "key": field.key,
                "label": field.label.clone().unwrap_or_else(|| field.key.clone()),
                "type": field.field_type.clone().unwrap_or_else(|| "string".into()),
                "widget": field_widget(field),
            })
        }).collect::<Vec<_>>()
    })
}

fn runtime_page(page: &SourceNoteTypePage) -> serde_json::Value {
    let kind = page.kind.as_deref().unwrap_or("content");
    match kind {
        "built_in" => json!({
            "id": page.id,
            "label": page.label.clone().unwrap_or_else(|| page.id.clone()),
            "kind": "built_in",
            "source": page.source.clone().unwrap_or_else(|| "connections".into()),
        }),
        "extension" => json!({
            "id": page.id,
            "label": page.label.clone().unwrap_or_else(|| page.id.clone()),
            "kind": "extension",
            "extension_id": page.source.clone().unwrap_or_default(),
        }),
        _ => json!({
            "id": page.id,
            "label": page.label.clone().unwrap_or_else(|| page.id.clone()),
            "kind": "content",
            "blocks": [
                {
                    "type": "field_group",
                    "label": page.label.clone().unwrap_or_else(|| page.id.clone()),
                    "fields": page.fields,
                }
            ]
        }),
    }
}

pub fn runtime_layout_json(note_type: &SourceNoteType) -> serde_json::Value {
    let mut pages = Vec::new();
    for page in &note_type.pages {
        pages.push(runtime_page(page));
    }
    json!({
        "version": 1,
        "pages": pages,
        "metadata": note_type.metadata.as_ref().map(toml_value_to_json).unwrap_or_else(|| json!({})),
    })
}
