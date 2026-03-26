use std::collections::{hash_map::DefaultHasher, BTreeMap, HashMap, HashSet};
use std::f64::consts::PI;
use std::hash::{Hash, Hasher};

use reqwest::Client;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::AppError;

#[taurpc::ipc_type]
pub struct AnkiNoteTypeFields {
    pub note_type: String,
    pub fields: Vec<String>,
}

#[taurpc::ipc_type]
pub struct AnkiOverviewFieldMapping {
    pub expression: Option<String>,
    pub reading: Option<String>,
    pub sentence: Option<String>,
    pub definition: Option<String>,
    pub context: Option<String>,
}

#[taurpc::ipc_type]
pub struct AnkiNoteModelMapping {
    pub note_type: String,
    pub title_fields: Vec<String>,
    pub overview_fields: AnkiOverviewFieldMapping,
    pub rendered_front_fields: Vec<String>,
    pub rendered_back_fields: Vec<String>,
    pub css_mode: Option<String>,
}

#[taurpc::ipc_type]
pub struct AnkiConnectPackSourceInput {
    pub id: String,
    pub name: String,
    pub deck_name: String,
    pub enabled: bool,
    pub anki_base_url: Option<String>,
    pub grouping_tag_prefix: String,
    pub include_media: bool,
    pub enforce_own_styles: bool,
    pub note_model_mappings: Option<Vec<AnkiNoteModelMapping>>,
}

#[taurpc::ipc_type]
pub struct AnkiDeckInspectInput {
    pub deck_name: String,
    pub anki_base_url: Option<String>,
}

#[taurpc::ipc_type]
pub struct AnkiDeckProbe {
    pub deck_name: String,
    pub card_count: u32,
    pub suggested_id: String,
    pub suggested_name: String,
    pub available_tags: Vec<String>,
    pub available_note_types: Vec<String>,
    pub available_fields: Vec<String>,
    pub note_type_fields: Vec<AnkiNoteTypeFields>,
}

#[derive(Clone)]
pub struct AnkiImportConfig {
    pub source_id: String,
    pub source_name: String,
    pub deck_name: String,
    pub anki_base_url: String,
    pub grouping_tag_prefix: String,
    pub include_media: bool,
    pub enforce_own_styles: bool,
    pub note_model_mappings: Vec<AnkiNoteModelMapping>,
}

#[derive(Debug, Deserialize)]
struct AnkiEnvelope<T> {
    result: Option<T>,
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct AnkiCardInfo {
    #[serde(rename = "cardId")]
    card_id: i64,
    #[serde(rename = "note")]
    note_id: i64,
    #[serde(rename = "deckName")]
    deck_name: String,
    #[serde(rename = "modelName")]
    model_name: String,
    #[serde(default)]
    tags: Vec<String>,
    #[serde(default)]
    fields: BTreeMap<String, AnkiFieldValue>,
    #[serde(default)]
    question: String,
    #[serde(default)]
    answer: String,
    #[serde(default)]
    template: Option<String>,
    #[serde(default)]
    ord: Option<i64>,
    #[serde(default)]
    due: Option<i64>,
    #[serde(default)]
    interval: Option<i64>,
    #[serde(default)]
    css: String,
}

#[derive(Debug, Deserialize)]
struct AnkiFieldValue {
    value: String,
}

#[derive(Clone)]
struct ImportedCard {
    card_id: i64,
    note_id: i64,
    deck_name: String,
    note_type: String,
    template_name: String,
    tags: Vec<String>,
    title: String,
    expression: String,
    reading: String,
    sentence: String,
    definition: String,
    context: String,
    rendered_front: String,
    rendered_back: String,
    rendered_css: String,
    due_hint: String,
    import_group: String,
    raw_group_tag: String,
    raw_tag_set: HashSet<String>,
}

fn sanitize_base_url(value: Option<&str>) -> String {
    value
        .unwrap_or("http://127.0.0.1:8765")
        .trim()
        .trim_end_matches('/')
        .to_string()
}

fn slugify(value: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;
    for ch in value.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() {
            slug.push(lower);
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "anki-import".into()
    } else {
        trimmed
    }
}

fn deck_world_id(deck_name: &str) -> String {
    format!("anki-{}", slugify(&deck_name.replace("::", "-")))
}

fn deck_world_name(deck_name: &str) -> String {
    format!("Anki: {deck_name}")
}

fn build_due_hint(card: &AnkiCardInfo) -> String {
    match (card.interval, card.due) {
        (Some(interval), Some(due)) => format!("interval {interval} · due {due}"),
        (Some(interval), None) => format!("interval {interval}"),
        (None, Some(due)) => format!("due {due}"),
        _ => String::new(),
    }
}

fn strip_html(input: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut previous_space = false;
    for ch in input.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            '&' => {}
            _ => {
                let is_space = ch.is_whitespace();
                if is_space {
                    if !previous_space {
                        out.push(' ');
                    }
                } else {
                    out.push(ch);
                }
                previous_space = is_space;
            }
        }
    }
    out.replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn truncate_label(value: &str, max: usize) -> String {
    let mut chars = value.chars();
    let truncated: String = chars.by_ref().take(max).collect();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

fn strip_block_tags(input: &str, tag: &str) -> String {
    let open_tag = format!("<{tag}");
    let close_tag = format!("</{tag}>");
    let mut output = input.to_string();
    loop {
        let Some(start) = output.to_ascii_lowercase().find(&open_tag) else {
            break;
        };
        let lower = output.to_ascii_lowercase();
        let Some(end_rel) = lower[start..].find(&close_tag) else {
            output.truncate(start);
            break;
        };
        let end = start + end_rel + close_tag.len();
        output.replace_range(start..end, "");
    }
    output
}

fn strip_inline_event_handlers(input: &str) -> String {
    let mut output = input.to_string();
    for attr in [
        "onabort", "onanimationend", "onanimationiteration", "onanimationstart", "onauxclick",
        "onbeforeinput", "onblur", "oncanplay", "oncanplaythrough", "onchange", "onclick",
        "onclose", "oncontextmenu", "oncopy", "oncuechange", "oncut", "ondblclick", "ondrag",
        "ondragend", "ondragenter", "ondragleave", "ondragover", "ondragstart", "ondrop",
        "ondurationchange", "onended", "onerror", "onfocus", "onfocusin", "onfocusout",
        "onformdata", "oninput", "oninvalid", "onkeydown", "onkeypress", "onkeyup",
        "onload", "onloadeddata", "onloadedmetadata", "onloadstart", "onmousedown",
        "onmouseenter", "onmouseleave", "onmousemove", "onmouseout", "onmouseover",
        "onmouseup", "onpaste", "onpause", "onplay", "onplaying", "onprogress",
        "onratechange", "onreset", "onresize", "onscroll", "onseeked", "onseeking",
        "onselect", "onstalled", "onsubmit", "onsuspend", "ontimeupdate", "ontoggle",
        "ontouchcancel", "ontouchend", "ontouchmove", "ontouchstart", "ontransitionend",
        "onvolumechange", "onwaiting", "onwheel",
    ] {
        loop {
            let lower = output.to_ascii_lowercase();
            let Some(start) = lower.find(attr) else {
                break;
            };
            let remainder = &output[start + attr.len()..];
            let trimmed = remainder.trim_start();
            if !trimmed.starts_with('=') {
                break;
            }
            let consumed_ws = remainder.len() - trimmed.len();
            let value = &trimmed[1..].trim_start();
            let consumed_eq_ws = trimmed[1..].len() - value.len();
            let remove_start = start;
            let remove_end = if let Some(quote) = value.chars().next() {
                if quote == '"' || quote == '\'' {
                    let quote_len = quote.len_utf8();
                    if let Some(end_rel) = value[quote_len..].find(quote) {
                        start + attr.len() + consumed_ws + 1 + consumed_eq_ws + quote_len + end_rel + quote_len
                    } else {
                        start + attr.len() + consumed_ws + 1 + consumed_eq_ws + value.len()
                    }
                } else {
                    let end_rel = value.find(|ch: char| ch.is_whitespace() || ch == '>').unwrap_or(value.len());
                    start + attr.len() + consumed_ws + 1 + consumed_eq_ws + end_rel
                }
            } else {
                start + attr.len()
            };
            output.replace_range(remove_start..remove_end, "");
        }
    }
    output
}

fn neutralize_script_protocols(input: &str) -> String {
    input
        .replace("javascript:", "#blocked-script:")
        .replace("JavaScript:", "#blocked-script:")
        .replace("JAVASCRIPT:", "#blocked-script:")
}

fn sanitize_html_block(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut output = strip_block_tags(trimmed, "script");
    output = strip_block_tags(&output, "style");
    output = output
        .replace("<script", "&lt;script")
        .replace("<style", "&lt;style");

    if output.contains("[sound:") {
        output = output.replace("[sound:", "[audio: ");
    }
    if output.contains("<img") {
        output = format!("{output}<p>[image omitted]</p>");
    }
    if output.contains("<audio") || output.contains("<video") {
        output = format!("{output}<p>[media omitted]</p>");
    }
    output
}

fn media_mime_type(filename: &str) -> &'static str {
    let extension = filename
        .rsplit('.')
        .next()
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_default();
    match extension.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        "svg" => "image/svg+xml",
        "mp3" => "audio/mpeg",
        "ogg" => "audio/ogg",
        "wav" => "audio/wav",
        "m4a" => "audio/mp4",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        _ => "application/octet-stream",
    }
}

fn extract_sound_refs(input: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut offset = 0;
    while let Some(start) = input[offset..].find("[sound:") {
        let absolute_start = offset + start + "[sound:".len();
        if let Some(end) = input[absolute_start..].find(']') {
            let filename = input[absolute_start..absolute_start + end].trim();
            if !filename.is_empty() {
                refs.push(filename.to_string());
            }
            offset = absolute_start + end + 1;
        } else {
            break;
        }
    }
    refs
}

fn extract_media_src_refs(input: &str, tag: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let lower = input.to_ascii_lowercase();
    let open = format!("<{tag}");
    let mut offset = 0;
    while let Some(start) = lower[offset..].find(&open) {
        let absolute_start = offset + start;
        let Some(close_rel) = lower[absolute_start..].find('>') else {
            break;
        };
        let end = absolute_start + close_rel + 1;
        let fragment = &input[absolute_start..end];
        let fragment_lower = fragment.to_ascii_lowercase();
        let Some(src_index) = fragment_lower.find("src=") else {
            offset = end;
            continue;
        };
        let value_start = src_index + 4;
        let quote = fragment[value_start..].chars().next().unwrap_or('"');
        if quote != '"' && quote != '\'' {
            offset = end;
            continue;
        }
        let content_start = value_start + quote.len_utf8();
        if let Some(close_quote) = fragment[content_start..].find(quote) {
            let src = fragment[content_start..content_start + close_quote].trim();
            if !src.is_empty() {
                refs.push(src.to_string());
            }
        }
        offset = end;
    }
    refs
}

async fn retrieve_media_data_uri(base_url: &str, filename: &str) -> Result<Option<String>, AppError> {
    if filename.trim().is_empty() {
        return Ok(None);
    }
    let result: Option<String> = anki_request(
        base_url,
        "retrieveMediaFile",
        json!({ "filename": filename }),
    )
    .await?;
    Ok(result.map(|base64| format!("data:{};base64,{}", media_mime_type(filename), base64)))
}

async fn rewrite_media_refs(input: &str, base_url: &str, include_media: bool) -> Result<String, AppError> {
    let mut output = input.to_string();

    for filename in extract_sound_refs(input) {
        let replacement = if include_media {
            if let Some(data_uri) = retrieve_media_data_uri(base_url, &filename).await? {
                format!(
                    "<audio controls preload=\"none\" src=\"{data_uri}\"></audio>"
                )
            } else {
                format!("<p>[missing audio: {filename}]</p>")
            }
        } else {
            format!("<p>[audio omitted: {filename}]</p>")
        };
        output = output.replace(&format!("[sound:{filename}]"), &replacement);
    }

    for filename in extract_media_src_refs(input, "img") {
        if filename.starts_with("http://") || filename.starts_with("https://") || filename.starts_with("data:") {
            continue;
        }
        let replacement = if include_media {
            if let Some(data_uri) = retrieve_media_data_uri(base_url, &filename).await? {
                data_uri
            } else {
                String::from("data:,missing-image")
            }
        } else {
            String::from("data:,image-omitted")
        };
        output = output.replace(&format!("src=\"{filename}\""), &format!("src=\"{replacement}\""));
        output = output.replace(&format!("src='{filename}'"), &format!("src='{replacement}'"));
        if !include_media && replacement == "data:,image-omitted" {
            output.push_str(&format!("<p>[image omitted: {filename}]</p>"));
        }
    }

    for filename in extract_media_src_refs(input, "audio") {
        if filename.starts_with("http://") || filename.starts_with("https://") || filename.starts_with("data:") {
            continue;
        }
        let replacement = if include_media {
            if let Some(data_uri) = retrieve_media_data_uri(base_url, &filename).await? {
                data_uri
            } else {
                String::from("data:,missing-audio")
            }
        } else {
            String::from("data:,audio-omitted")
        };
        output = output.replace(&format!("src=\"{filename}\""), &format!("src=\"{replacement}\""));
        output = output.replace(&format!("src='{filename}'"), &format!("src='{replacement}'"));
    }

    Ok(output)
}

fn sanitize_rendered_html(input: &str, _card_css: &str, enforce_own_styles: bool) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut output = strip_block_tags(trimmed, "script");
    output = strip_inline_event_handlers(&output);
    output = neutralize_script_protocols(&output);
    output = output.replace("<script", "&lt;script");
    if !enforce_own_styles {
        output = strip_block_tags(&output, "style");
        output = output.replace("<style", "&lt;style");
    }

    let mode_class = if enforce_own_styles { "anki-rich-card source-mode" } else { "anki-rich-card clean-mode" };
    format!("<div class=\"{mode_class}\">{output}</div>")
}

fn field_value<'a>(fields: &'a BTreeMap<String, AnkiFieldValue>, key: &str) -> Option<&'a str> {
    fields
        .get(key)
        .map(|value| value.value.trim())
        .filter(|value| !value.is_empty())
}

fn first_matching_field<'a>(fields: &'a BTreeMap<String, AnkiFieldValue>, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| field_value(fields, key))
}

fn first_present_field<'a>(fields: &'a BTreeMap<String, AnkiFieldValue>, keys: &[String]) -> Option<&'a str> {
    keys.iter().find_map(|key| field_value(fields, key))
}

fn find_note_model_mapping<'a>(config: &'a AnkiImportConfig, note_type: &str) -> Option<&'a AnkiNoteModelMapping> {
    config
        .note_model_mappings
        .iter()
        .find(|mapping| mapping.note_type.eq_ignore_ascii_case(note_type))
}

fn preferred_mapping_for_note_type(note_type: &str) -> Option<AnkiNoteModelMapping> {
    if !note_type.eq_ignore_ascii_case("Kiku") {
        return None;
    }
    Some(AnkiNoteModelMapping {
        note_type: "Kiku".into(),
        title_fields: vec!["Expression".into(), "ExpressionReading".into(), "MainDefinition".into()],
        overview_fields: AnkiOverviewFieldMapping {
            expression: Some("Expression".into()),
            reading: Some("ExpressionReading".into()),
            sentence: Some("Sentence".into()),
            definition: Some("MainDefinition".into()),
            context: Some("MiscInfo".into()),
        },
        rendered_front_fields: vec![
            "Expression".into(),
            "ExpressionFurigana".into(),
            "ExpressionReading".into(),
            "Sentence".into(),
            "SentenceFurigana".into(),
            "Picture".into(),
            "ExpressionAudio".into(),
            "SentenceAudio".into(),
        ],
        rendered_back_fields: vec![
            "MainDefinition".into(),
            "Glossary".into(),
            "MiscInfo".into(),
        ],
        css_mode: Some("source".into()),
    })
}

fn clean_text_value(value: Option<&str>) -> String {
    value
        .map(sanitize_html_block)
        .map(|html| strip_html(&html))
        .unwrap_or_default()
}

fn html_or_empty(value: Option<&str>) -> String {
    value.map(|raw| raw.trim().to_string()).unwrap_or_default()
}

fn derive_card_content(card: &AnkiCardInfo, config: &AnkiImportConfig) -> (String, String, String, String, String, String) {
    let mapping = find_note_model_mapping(config, &card.model_name)
        .cloned()
        .or_else(|| preferred_mapping_for_note_type(&card.model_name));
    let expression = mapping
        .as_ref()
        .and_then(|mapping| mapping.overview_fields.expression.as_deref())
        .and_then(|key| field_value(&card.fields, key))
        .or_else(|| first_matching_field(
            &card.fields,
            &["Expression", "Front", "Word", "Vocabulary", "Kanji", "Term", "Question"],
        ));
    let reading = mapping
        .as_ref()
        .and_then(|mapping| mapping.overview_fields.reading.as_deref())
        .and_then(|key| field_value(&card.fields, key))
        .or_else(|| first_matching_field(
            &card.fields,
            &["ExpressionReading", "Reading", "Kana", "Pronunciation", "ExpressionFurigana"],
        ));
    let sentence = mapping
        .as_ref()
        .and_then(|mapping| mapping.overview_fields.sentence.as_deref())
        .and_then(|key| field_value(&card.fields, key))
        .or_else(|| first_matching_field(
            &card.fields,
            &["Sentence", "SentenceFurigana", "Example", "ExampleSentence"],
        ));
    let definition = mapping
        .as_ref()
        .and_then(|mapping| mapping.overview_fields.definition.as_deref())
        .and_then(|key| field_value(&card.fields, key))
        .or_else(|| first_matching_field(
            &card.fields,
            &["MainDefinition", "Glossary", "Back", "Definition", "Meaning", "Translation", "Answer"],
        ));
    let misc_info = mapping
        .as_ref()
        .and_then(|mapping| mapping.overview_fields.context.as_deref())
        .and_then(|key| field_value(&card.fields, key))
        .or_else(|| first_matching_field(&card.fields, &["MiscInfo", "Context", "Source", "Tags"]));
    let fallback_front = if !card.question.trim().is_empty() {
        Some(card.question.as_str())
    } else {
        card.fields
            .values()
            .map(|value| value.value.as_str())
            .find(|value| !value.trim().is_empty())
    };
    let fallback_back = if !card.answer.trim().is_empty() {
        Some(card.answer.as_str())
    } else {
        card.fields
            .values()
            .skip(1)
            .map(|value| value.value.as_str())
            .find(|value| !value.trim().is_empty())
    };

    let expression_text = clean_text_value(expression.or(fallback_front));
    let reading_text = clean_text_value(reading);
    let sentence_text = clean_text_value(sentence);
    let definition_text = clean_text_value(definition.or(fallback_back));
    let context_text = clean_text_value(misc_info);

    let mapped_title = mapping
        .as_ref()
        .and_then(|mapping| first_present_field(&card.fields, &mapping.title_fields))
        .map(|value| clean_text_value(Some(value)))
        .unwrap_or_default();

    let title = if !mapped_title.is_empty() {
        truncate_label(&mapped_title, 48)
    } else if !expression_text.is_empty() {
        truncate_label(&expression_text, 48)
    } else if !reading_text.is_empty() {
        truncate_label(&reading_text, 48)
    } else if !definition_text.is_empty() {
        truncate_label(&definition_text, 48)
    } else {
        format!("Card {}", card.card_id)
    };

    (title, expression_text, reading_text, sentence_text, definition_text, context_text)
}

async fn build_rendered_views(card: &AnkiCardInfo, config: &AnkiImportConfig) -> Result<(String, String, String), AppError> {
    let mapping = find_note_model_mapping(config, &card.model_name)
        .cloned()
        .or_else(|| preferred_mapping_for_note_type(&card.model_name));

    let mut front = String::new();
    if let Some(mapping) = &mapping {
        for field_name in &mapping.rendered_front_fields {
            if let Some(value) = field_value(&card.fields, field_name) {
                front.push_str(&format!("<section><h3>{}</h3>{}</section>", field_name, html_or_empty(Some(value))));
            }
        }
    } else {
        let expression = first_matching_field(
            &card.fields,
            &["Expression", "Front", "Word", "Vocabulary", "Kanji", "Term", "Question"],
        );
        let reading = first_matching_field(
            &card.fields,
            &["ExpressionReading", "Reading", "Kana", "Pronunciation", "ExpressionFurigana"],
        );
        let sentence = first_matching_field(
            &card.fields,
            &["Sentence", "SentenceFurigana", "Example", "ExampleSentence"],
        );
        let picture = first_matching_field(&card.fields, &["Picture", "Image"]);
        let expression_audio = first_matching_field(&card.fields, &["ExpressionAudio", "Audio"]);
        let sentence_audio = first_matching_field(&card.fields, &["SentenceAudio"]);
        if !expression.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Expression</h3>{}</section>", html_or_empty(expression)));
        }
        if !reading.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Reading</h3>{}</section>", html_or_empty(reading)));
        }
        if !sentence.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Sentence</h3>{}</section>", html_or_empty(sentence)));
        }
        if !picture.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Picture</h3>{}</section>", html_or_empty(picture)));
        }
        if !expression_audio.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Expression Audio</h3>{}</section>", html_or_empty(expression_audio)));
        }
        if !sentence_audio.unwrap_or("").trim().is_empty() {
            front.push_str(&format!("<section><h3>Sentence Audio</h3>{}</section>", html_or_empty(sentence_audio)));
        }
    }
    if front.trim().is_empty() {
        front = card.question.trim().to_string();
    }

    let mut back = String::new();
    if let Some(mapping) = &mapping {
        for field_name in &mapping.rendered_back_fields {
            if let Some(value) = field_value(&card.fields, field_name) {
                back.push_str(&format!("<section><h3>{}</h3>{}</section>", field_name, html_or_empty(Some(value))));
            }
        }
    }
    let mut remaining_fields = Vec::new();
    for (name, field) in &card.fields {
        if ["Expression", "Front", "Word", "Vocabulary", "Kanji", "Term", "Question",
            "ExpressionReading", "Reading", "Kana", "Pronunciation", "ExpressionFurigana",
            "Sentence", "SentenceFurigana", "Example", "ExampleSentence",
            "MainDefinition", "Glossary", "Back", "Definition", "Meaning", "Translation", "Answer",
            "MiscInfo", "Context", "Source", "Tags", "Picture", "Image", "ExpressionAudio", "Audio", "SentenceAudio"]
            .contains(&name.as_str()) {
            if mapping.is_none() {
                continue;
            }
        }
        if mapping.as_ref().is_some_and(|mapping| {
            mapping.rendered_front_fields.iter().any(|field_name| field_name == name)
                || mapping.rendered_back_fields.iter().any(|field_name| field_name == name)
        }) {
            continue;
        }
        if field.value.trim().is_empty() {
            continue;
        }
        remaining_fields.push(format!(
            "<section><h3>{}</h3>{}</section>",
            name,
            sanitize_html_block(&field.value)
        ));
    }
    if !remaining_fields.is_empty() {
        back.push_str(&remaining_fields.join(""));
    }
    if back.trim().is_empty() {
        back = card.answer.trim().to_string();
    }

    let rewritten_front = rewrite_media_refs(&front, &config.anki_base_url, config.include_media).await?;
    let rewritten_back = rewrite_media_refs(&back, &config.anki_base_url, config.include_media).await?;
    let css_mode = mapping
        .as_ref()
        .and_then(|mapping| mapping.css_mode.as_deref())
        .unwrap_or(if config.enforce_own_styles { "source" } else { "clean" });
    let use_source_css = config.enforce_own_styles && css_mode.eq_ignore_ascii_case("source");
    Ok((
        sanitize_rendered_html(&rewritten_front, &card.css, use_source_css),
        sanitize_rendered_html(&rewritten_back, &card.css, use_source_css),
        if use_source_css { card.css.clone() } else { String::new() },
    ))
}

fn resolve_import_group(tags: &[String], grouping_tag_prefix: &str, deck_name: &str) -> (String, String) {
    let prefix = grouping_tag_prefix.trim().to_ascii_lowercase();
    for tag in tags {
        let lower = tag.to_ascii_lowercase();
        if lower.starts_with(&prefix) {
            let suffix = tag[prefix.len()..].trim_matches(':').trim();
            if !suffix.is_empty() {
                let group = slugify(suffix);
                return (group.clone(), format!("group:{group}"));
            }
        }
    }

    let fallback = deck_name
        .split("::")
        .last()
        .map(slugify)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "deck".into());
    (fallback.clone(), format!("group:{fallback}"))
}

fn hash_unit<T: Hash>(value: &T) -> f64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    let raw = hasher.finish() % 1_000_000;
    raw as f64 / 1_000_000.0
}

fn build_positions(cards: &[ImportedCard]) -> HashMap<i64, Value> {
    let mut groups: BTreeMap<String, Vec<&ImportedCard>> = BTreeMap::new();
    for card in cards {
        groups.entry(card.import_group.clone()).or_default().push(card);
    }

    let total_groups = groups.len().max(1) as f64;
    let mut positions = HashMap::new();

    for (group_index, (group, entries)) in groups.into_iter().enumerate() {
        let group_seed = hash_unit(&group);
        let base_angle = ((group_index as f64) / total_groups) * PI * 2.0 + (group_seed - 0.5) * 0.9;
        let radius = 18.0 + group_seed * 11.0;
        let center_x = radius * base_angle.cos();
        let center_z = radius * base_angle.sin();
        let center_y = -7.0 + group_seed * 14.0;

        let mut sorted = entries;
        sorted.sort_by_key(|card| card.card_id);
        for (local_index, card) in sorted.into_iter().enumerate() {
            let local_seed = hash_unit(&(card.card_id, &card.import_group));
            let local_angle = (local_index as f64) * 0.58 + local_seed * 0.7;
            let local_radius = 2.6 + (local_index as f64) * 0.34 + local_seed * 1.8;
            let x = center_x + local_radius * local_angle.cos() + (local_seed - 0.5) * 2.2;
            let y = center_y + ((local_index % 5) as f64 - 2.0) * 2.1 + (local_seed - 0.5) * 5.8;
            let z = center_z + local_radius * local_angle.sin() + (0.5 - local_seed) * 2.6;
            positions.insert(
                card.card_id,
                json!({
                    "x": (x * 100.0).round() / 100.0,
                    "y": (y * 100.0).round() / 100.0,
                    "z": (z * 100.0).round() / 100.0,
                }),
            );
        }
    }

    positions
}

async fn anki_request<T: for<'de> Deserialize<'de>>(base_url: &str, action: &str, params: Value) -> Result<T, AppError> {
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(12))
        .build()
        .map_err(|e| AppError::Other(e.to_string()))?;
    let url = format!("{}/", sanitize_base_url(Some(base_url)));
    let payload = json!({
        "action": action,
        "version": 6,
        "params": params,
    });
    let envelope = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| AppError::Other(format!("Failed to reach AnkiConnect: {e}")))?
        .error_for_status()
        .map_err(|e| AppError::Other(format!("AnkiConnect returned an HTTP error: {e}")))?
        .json::<AnkiEnvelope<T>>()
        .await
        .map_err(|e| AppError::Other(format!("Invalid AnkiConnect response: {e}")))?;

    if let Some(error) = envelope.error.filter(|value| !value.trim().is_empty()) {
        return Err(AppError::Other(format!("AnkiConnect error: {error}")));
    }
    envelope
        .result
        .ok_or_else(|| AppError::Other(format!("AnkiConnect action '{action}' returned no result")))
}

pub async fn list_decks(base_url: Option<&str>) -> Result<Vec<String>, AppError> {
    let mut decks: Vec<String> = anki_request(&sanitize_base_url(base_url), "deckNames", json!({})).await?;
    decks.sort();
    Ok(decks)
}

async fn fetch_cards(base_url: &str, deck_name: &str) -> Result<Vec<AnkiCardInfo>, AppError> {
    let escaped_deck = deck_name.replace('"', "\\\"");
    let query = format!("deck:\"{escaped_deck}\"");
    let card_ids: Vec<i64> = anki_request(base_url, "findCards", json!({ "query": query })).await?;
    if card_ids.is_empty() {
        return Ok(Vec::new());
    }
    anki_request(base_url, "cardsInfo", json!({ "cards": card_ids })).await
}

pub async fn inspect_deck(input: &AnkiDeckInspectInput) -> Result<AnkiDeckProbe, AppError> {
    let base_url = sanitize_base_url(input.anki_base_url.as_deref());
    let cards = fetch_cards(&base_url, input.deck_name.trim()).await?;
    let mut tags = BTreeMap::<String, ()>::new();
    let mut note_types = BTreeMap::<String, ()>::new();
    let mut fields = BTreeMap::<String, ()>::new();
    let mut note_type_fields = BTreeMap::<String, BTreeMap<String, ()>>::new();

    for card in &cards {
        note_types.insert(card.model_name.clone(), ());
        for tag in &card.tags {
            tags.insert(tag.clone(), ());
        }
        for field_name in card.fields.keys() {
            fields.insert(field_name.clone(), ());
            note_type_fields
                .entry(card.model_name.clone())
                .or_default()
                .insert(field_name.clone(), ());
        }
    }

    Ok(AnkiDeckProbe {
        deck_name: input.deck_name.trim().to_string(),
        card_count: cards.len() as u32,
        suggested_id: deck_world_id(input.deck_name.trim()),
        suggested_name: deck_world_name(input.deck_name.trim()),
        available_tags: tags.into_keys().take(24).collect(),
        available_note_types: note_types.into_keys().collect(),
        available_fields: fields.into_keys().collect(),
        note_type_fields: note_type_fields
            .into_iter()
            .map(|(note_type, fields)| AnkiNoteTypeFields {
                note_type,
                fields: fields.into_keys().collect(),
            })
            .collect(),
    })
}

async fn imported_cards(cards: Vec<AnkiCardInfo>, config: &AnkiImportConfig) -> Result<Vec<ImportedCard>, AppError> {
    let mut imported = Vec::with_capacity(cards.len());
    for card in cards {
        let (title, expression, reading, sentence, definition, context) = derive_card_content(&card, config);
        let (import_group, raw_group_tag) =
            resolve_import_group(&card.tags, &config.grouping_tag_prefix, &card.deck_name);
        let raw_tag_set = card.tags.iter().map(|tag| tag.to_ascii_lowercase()).collect::<HashSet<_>>();
        let due_hint = build_due_hint(&card);
        let (rendered_front, rendered_back, rendered_css) = build_rendered_views(&card, config).await?;
        imported.push(ImportedCard {
            card_id: card.card_id,
            note_id: card.note_id,
            deck_name: card.deck_name,
            note_type: card.model_name,
            template_name: card
                .template
                .clone()
                .unwrap_or_else(|| format!("Card {}", card.ord.unwrap_or(0) + 1)),
            tags: card.tags,
            title,
            expression,
            reading,
            sentence,
            definition,
            context,
            rendered_front,
            rendered_back,
            rendered_css,
            due_hint,
            import_group,
            raw_group_tag,
            raw_tag_set,
        });
    }
    Ok(imported)
}

fn relation_kinds() -> Value {
    json!([
        { "id": "next-card", "label": "Next Card", "directed": true, "default_weight": 1.0, "metadata": {} },
        { "id": "same-group", "label": "Same Group", "directed": false, "default_weight": 0.8, "metadata": {} },
        { "id": "same-note", "label": "Same Note", "directed": false, "default_weight": 0.8, "metadata": {} },
        { "id": "same-template", "label": "Same Template", "directed": false, "default_weight": 0.7, "metadata": {} },
        { "id": "memory-anchor", "label": "Memory Anchor", "directed": false, "default_weight": 0.6, "metadata": {} }
    ])
}

fn connection_layers() -> Value {
    json!([
        { "id": "path", "name": "Path", "display_order": 0, "metadata": { "edge_style": { "color": "#7dd3fc", "width": 5.4, "shape": "straight" } } },
        { "id": "group", "name": "Groups", "display_order": 1, "metadata": { "edge_style": { "color": "#22c55e", "width": 4.2, "shape": "arc" } } },
        { "id": "source", "name": "Source", "display_order": 2, "metadata": { "edge_style": { "color": "#f59e0b", "width": 3.8, "shape": "arc", "dash_size": 0.52 } } },
        { "id": "memory", "name": "Memory", "display_order": 3, "metadata": { "edge_style": { "color": "#d946ef", "width": 3.6, "shape": "arc", "dash_size": 0.14 } } }
    ])
}

fn generated_note_type() -> Value {
    json!([{
        "id": "anki-card-v1",
        "name": "Anki Card",
        "fields": [
            "Expression", "Reading", "Sentence", "Definition", "Context", "RenderedFrontHtml", "RenderedBackHtml", "RenderedCss", "RenderedThemeMode", "RenderedMediaMode", "Deck", "NoteType", "CardTemplate", "Tags",
            "SourceCardId", "SourceNoteId", "DueHint", "ImportGroup"
        ],
        "schema_json": {
            "version": 1,
            "fields": [
                { "key": "Expression", "label": "Expression", "type": "string", "widget": "text" },
                { "key": "Reading", "label": "Reading", "type": "string", "widget": "text" },
                { "key": "Sentence", "label": "Sentence", "type": "string", "widget": "long_text" },
                { "key": "Definition", "label": "Definition", "type": "string", "widget": "long_text" },
                { "key": "Context", "label": "Context", "type": "string", "widget": "long_text" },
                { "key": "RenderedFrontHtml", "label": "Rendered Front HTML", "type": "string", "widget": "html" },
                { "key": "RenderedBackHtml", "label": "Rendered Back HTML", "type": "string", "widget": "html" },
                { "key": "RenderedCss", "label": "Rendered CSS", "type": "string", "widget": "long_text" },
                { "key": "RenderedThemeMode", "label": "Rendered Theme Mode", "type": "string", "widget": "text" },
                { "key": "RenderedMediaMode", "label": "Rendered Media Mode", "type": "string", "widget": "text" },
                { "key": "Deck", "label": "Deck", "type": "string", "widget": "text" },
                { "key": "NoteType", "label": "Note Type", "type": "string", "widget": "text" },
                { "key": "CardTemplate", "label": "Card Template", "type": "string", "widget": "text" },
                { "key": "Tags", "label": "Tags", "type": "string", "widget": "long_text" },
                { "key": "SourceCardId", "label": "Source Card Id", "type": "string", "widget": "text" },
                { "key": "SourceNoteId", "label": "Source Note Id", "type": "string", "widget": "text" },
                { "key": "DueHint", "label": "Due Hint", "type": "string", "widget": "text" },
                { "key": "ImportGroup", "label": "Import Group", "type": "string", "widget": "text" }
            ]
        },
        "layout_json": {
            "version": 1,
            "pages": [
                {
                    "id": "overview",
                    "label": "Overview",
                    "kind": "content",
                    "sections": [
                        { "id": "main", "label": "Main", "items": [ { "field": "Expression" }, { "field": "Reading" }, { "field": "Sentence" } ] }
                    ]
                },
                {
                    "id": "card",
                    "label": "Card",
                    "kind": "card",
                    "source": "anki-card",
                    "card_slot": "split"
                },
                {
                    "id": "source",
                    "label": "Source",
                    "kind": "content",
                    "sections": [
                        { "id": "source-main", "label": "Source", "items": [ { "field": "Deck" }, { "field": "ImportGroup" }, { "field": "NoteType" }, { "field": "CardTemplate" }, { "field": "Tags" }, { "field": "SourceCardId" }, { "field": "SourceNoteId" }, { "field": "DueHint" } ] }
                    ]
                },
                { "id": "connections", "label": "Connections", "kind": "built_in", "source": "connections" }
            ]
        },
        "metadata": {
            "source": "anki-connect",
            "viewer_mode": "anki-card",
            "card_surface": {
                "type": "static-rich",
                "supports_css": true,
                "supports_media": true,
                "supports_scripts": false
            }
        },
        "is_default": true
    }])
}

fn edge(id: String, source_id: String, target_id: String, relation_id: &str, connection_layer: &str, weight: f64) -> Value {
    json!({
        "id": id,
        "source_id": source_id,
        "target_id": target_id,
        "relation_id": relation_id,
        "edge_type": "Semantic",
        "weight": weight,
        "connection_layer_membership": [connection_layer],
        "metadata": {}
    })
}

fn chain_edges(cards: &[&ImportedCard], relation_id: &str, layer: &str, weight: f64, prefix: &str) -> Vec<Value> {
    let mut out = Vec::new();
    for pair in cards.windows(2) {
        let source = pair[0];
        let target = pair[1];
        out.push(edge(
            format!("{prefix}-{}-{}", source.card_id, target.card_id),
            format!("anki-card-{}", source.card_id),
            format!("anki-card-{}", target.card_id),
            relation_id,
            layer,
            weight,
        ));
    }
    out
}

pub async fn generate_pack_json(config: &AnkiImportConfig) -> Result<String, AppError> {
    let cards = fetch_cards(&config.anki_base_url, &config.deck_name).await?;
    if cards.is_empty() {
        return Err(AppError::Other(format!("Anki deck '{}' has no cards", config.deck_name)));
    }

    let imported = imported_cards(cards, config).await?;
    let positions = build_positions(&imported);

    let nodes = imported
        .iter()
        .map(|card| {
            let mut tags = card.tags.clone();
            tags.push(format!("deck:{}", slugify(&card.deck_name.replace("::", "-"))));
            tags.push("source:anki".into());
            tags.push(card.raw_group_tag.clone());
            tags.push(format!("cluster:{}", card.import_group));
            tags.push(format!("anki-note-type:{}", slugify(&card.note_type)));
            tags.push(format!("anki-card-template:{}", slugify(&card.template_name)));
            json!({
                "id": format!("anki-card-{}", card.card_id),
                "title": card.title,
                "node_type": "vocab",
                "note_type_id": "anki-card-v1",
                "note_fields": {
                    "Expression": card.expression,
                    "Reading": card.reading,
                    "Sentence": card.sentence,
                    "Definition": card.definition,
                    "Context": card.context,
                    "RenderedFrontHtml": card.rendered_front,
                    "RenderedBackHtml": card.rendered_back,
                    "RenderedCss": card.rendered_css,
                    "RenderedThemeMode": if config.enforce_own_styles { "source" } else { "clean" },
                    "RenderedMediaMode": if config.include_media { "inline" } else { "omitted" },
                    "Deck": card.deck_name,
                    "NoteType": card.note_type,
                    "CardTemplate": card.template_name,
                    "Tags": card.tags.join(", "),
                    "SourceCardId": card.card_id.to_string(),
                    "SourceNoteId": card.note_id.to_string(),
                    "DueHint": card.due_hint,
                    "ImportGroup": card.import_group,
                },
                "content_data": card.definition,
                "tags": tags,
                "weight": 1.0,
                "position": positions.get(&card.card_id).cloned().unwrap_or_else(|| json!({"x":0.0,"y":0.0,"z":0.0})),
                "layer_membership": ["main"],
                "metadata": {}
            })
        })
        .collect::<Vec<_>>();

    let mut edges = Vec::new();

    let mut path_cards = imported.iter().collect::<Vec<_>>();
    path_cards.sort_by(|a, b| {
        a.deck_name
            .cmp(&b.deck_name)
            .then(a.import_group.cmp(&b.import_group))
            .then(a.note_type.cmp(&b.note_type))
            .then(a.card_id.cmp(&b.card_id))
    });
    edges.extend(chain_edges(&path_cards, "next-card", "path", 0.9, "path"));

    let mut by_group = BTreeMap::<String, Vec<&ImportedCard>>::new();
    let mut by_note = BTreeMap::<i64, Vec<&ImportedCard>>::new();
    let mut by_template = BTreeMap::<String, Vec<&ImportedCard>>::new();
    for card in &imported {
        by_group.entry(card.import_group.clone()).or_default().push(card);
        by_note.entry(card.note_id).or_default().push(card);
        by_template
            .entry(format!("{}::{}", card.note_type, card.template_name))
            .or_default()
            .push(card);
    }

    for (group, mut group_cards) in by_group {
        group_cards.sort_by_key(|card| card.card_id);
        edges.extend(chain_edges(&group_cards, "same-group", "group", 0.78, &format!("group-{group}")));

        let len = group_cards.len();
        for index in 0..len {
            let source = group_cards[index];
            let mut added = 0;
            for target in group_cards.iter().skip(index + 1) {
                if added >= 2 {
                    break;
                }
                let overlap = source
                    .raw_tag_set
                    .intersection(&target.raw_tag_set)
                    .filter(|tag| !tag.starts_with(&config.grouping_tag_prefix.to_ascii_lowercase()))
                    .next()
                    .is_some();
                if overlap {
                    edges.push(edge(
                        format!("memory-{}-{}", source.card_id, target.card_id),
                        format!("anki-card-{}", source.card_id),
                        format!("anki-card-{}", target.card_id),
                        "memory-anchor",
                        "memory",
                        0.64,
                    ));
                    added += 1;
                }
            }
        }
    }

    for (note_id, mut note_cards) in by_note {
        if note_cards.len() < 2 {
            continue;
        }
        note_cards.sort_by_key(|card| card.card_id);
        edges.extend(chain_edges(&note_cards, "same-note", "source", 0.74, &format!("note-{note_id}")));
    }

    for (template_key, mut template_cards) in by_template {
        if template_cards.len() < 2 {
            continue;
        }
        template_cards.sort_by_key(|card| card.card_id);
        edges.extend(chain_edges(
            &template_cards,
            "same-template",
            "source",
            0.68,
            &format!("template-{}", slugify(&template_key)),
        ));
    }

    let pack = json!({
        "version": "2",
        "world": {
            "id": deck_world_id(&config.deck_name),
            "name": deck_world_name(&config.deck_name),
            "layout": {},
                "metadata": {
                    "source": "anki-connect",
                    "source_id": config.source_id,
                    "source_name": config.source_name,
                    "grouping_tag_prefix": config.grouping_tag_prefix,
                    "include_media": config.include_media,
                    "enforce_own_styles": config.enforce_own_styles,
                    "viewer_mode": "anki-card",
                    "focus_view": {
                    "rings": 2,
                    "ring_radius": 7.5,
                    "max_neighbors": 14
                },
                "inter_group_spacing": 28
            }
        },
        "note_types": generated_note_type(),
        "relation_kinds": relation_kinds(),
        "layers": [
            { "id": "main", "name": "Main", "display_order": 0, "node_filter": {}, "edge_filter": {}, "metadata": {} }
        ],
        "connection_layers": connection_layers(),
        "nodes": nodes,
        "edges": edges
    });

    serde_json::to_string_pretty(&pack).map_err(|e| AppError::Other(e.to_string()))
}
