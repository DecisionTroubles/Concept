use crate::error::AppError;

#[derive(Clone, Debug)]
pub struct MarkdownSection {
    pub heading: String,
    pub body: String,
}

#[derive(Clone, Debug)]
pub struct ParsedMarkdown {
    pub sections: Vec<MarkdownSection>,
    pub content_before_sections: String,
}

pub fn split_frontmatter(raw: &str) -> Result<(String, String), AppError> {
    let trimmed = raw.trim_start_matches('\u{feff}');
    if !trimmed.starts_with("+++") {
        return Err(AppError::Other("Node file is missing TOML frontmatter fence '+++'".into()));
    }

    let mut lines = trimmed.lines();
    let Some(first_line) = lines.next() else {
        return Err(AppError::Other("Node file is empty".into()));
    };
    if first_line.trim() != "+++" {
        return Err(AppError::Other("Node frontmatter must start with a standalone '+++' fence".into()));
    }

    let mut frontmatter = String::new();
    let mut body_lines = Vec::new();
    let mut in_frontmatter = true;
    for line in lines {
        if in_frontmatter && line.trim() == "+++" {
            in_frontmatter = false;
            continue;
        }
        if in_frontmatter {
            frontmatter.push_str(line);
            frontmatter.push('\n');
        } else {
            body_lines.push(line);
        }
    }

    if in_frontmatter {
        return Err(AppError::Other("Node frontmatter is missing closing '+++' fence".into()));
    }

    Ok((frontmatter, body_lines.join("\n").trim().to_string()))
}

pub fn parse_markdown_sections(body: &str) -> ParsedMarkdown {
    let mut sections = Vec::new();
    let mut prelude = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_body = Vec::new();
    let mut in_code_fence = false;

    for line in body.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_code_fence = !in_code_fence;
        }
        if !in_code_fence && trimmed.starts_with("# ") {
            if let Some(heading) = current_heading.take() {
                sections.push(MarkdownSection {
                    heading,
                    body: current_body.join("\n").trim().to_string(),
                });
                current_body.clear();
            }
            current_heading = Some(trimmed.trim_start_matches("# ").trim().to_string());
            continue;
        }

        if current_heading.is_some() {
            current_body.push(line);
        } else {
            prelude.push(line);
        }
    }

    if let Some(heading) = current_heading {
        sections.push(MarkdownSection {
            heading,
            body: current_body.join("\n").trim().to_string(),
        });
    }

    ParsedMarkdown {
        sections,
        content_before_sections: prelude.join("\n").trim().to_string(),
    }
}

pub fn normalized_heading(value: &str) -> String {
    value
        .to_ascii_lowercase()
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .collect()
}
