use crate::model::{DocPage, DocSection};
use std::path::Path;

pub fn parse_markdown_page(path_str: &str, content: &str) -> DocPage {
    let mut frontmatter_map = std::collections::HashMap::new();
    let body_start;

    let lines: Vec<&str> = content.lines().collect();
    if lines.first().map(|l| l.trim()) == Some("---") {
        let mut fm_end = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                fm_end = Some(i);
                break;
            }
            if let Some((k, v)) = line.split_once(':') {
                let key = k.trim().to_lowercase();
                let val = v.trim();
                frontmatter_map.insert(key, val.to_string());
            }
        }
        body_start = fm_end.map(|idx| idx + 1).unwrap_or(0);
    } else {
        body_start = 0;
    }

    let body_lines = &lines[body_start..];
    let body_text = body_lines.join("\n");

    let file_stem = Path::new(path_str)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("index")
        .to_string();

    let parent_dir = Path::new(path_str)
        .parent()
        .and_then(|p| p.file_name())
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_string();

    let default_category = match parent_dir.as_str() {
        "rules" => "rule",
        "guides" => "guide",
        _ => "general",
    };

    let id = frontmatter_map
        .get("id")
        .cloned()
        .unwrap_or_else(|| file_stem.clone());

    let category = frontmatter_map
        .get("category")
        .cloned()
        .unwrap_or_else(|| default_category.to_string());

    let aliases = if let Some(alias_str) = frontmatter_map.get("aliases") {
        parse_list_or_csv(alias_str)
    } else {
        Vec::new()
    };

    // Extract title from `# Title` or frontmatter
    let mut first_h1 = None;
    for line in body_lines {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            first_h1 = Some(trimmed[2..].trim().to_string());
            break;
        }
    }

    let title = frontmatter_map
        .get("title")
        .cloned()
        .or(first_h1)
        .unwrap_or_else(|| format_title(&id));

    // Extract summary (frontmatter, or first non-heading paragraph)
    let summary = if let Some(s) = frontmatter_map.get("summary") {
        s.clone()
    } else {
        extract_first_paragraph(body_lines)
    };

    // Extract sections by `## ` or `### `
    let sections = extract_sections(body_lines);

    DocPage {
        path: path_str.to_string(),
        id,
        title,
        summary,
        category,
        aliases,
        sections,
        raw_content: body_text,
    }
}

fn parse_list_or_csv(s: &str) -> Vec<String> {
    let trimmed = s.trim();
    let inner = if trimmed.starts_with('[') && trimmed.ends_with(']') {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };
    inner
        .split(',')
        .map(|item| item.trim().trim_matches('"').trim_matches('\'').to_string())
        .filter(|item| !item.is_empty())
        .collect()
}

fn format_title(id: &str) -> String {
    id.replace(['-', '_'], " ")
        .split_whitespace()
        .map(|w| {
            let mut c = w.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_first_paragraph(lines: &[&str]) -> String {
    let mut paragraph = Vec::new();
    let mut started = false;

    for line in lines {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            if started {
                break;
            }
            continue;
        }
        if trimmed.is_empty() {
            if started {
                break;
            }
            continue;
        }
        // Skip code fences or bullet lists if they are at the top
        if trimmed.starts_with("```") {
            break;
        }
        started = true;
        paragraph.push(trimmed);
    }

    paragraph.join(" ")
}

pub fn slugify(text: &str) -> String {
    text.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn extract_sections(lines: &[&str]) -> Vec<DocSection> {
    let mut sections = Vec::new();
    let mut current_heading = String::new();
    let mut current_level = 0;
    let mut current_lines = Vec::new();

    for line in lines {
        let trimmed = line.trim();
        let (level, heading_text) = if trimmed.starts_with("### ") {
            (3, trimmed[4..].trim())
        } else if trimmed.starts_with("## ") {
            (2, trimmed[3..].trim())
        } else {
            (0, "")
        };

        if level > 0 {
            if !current_heading.is_empty() {
                let anchor = slugify(&current_heading);
                sections.push(DocSection {
                    heading: current_heading.clone(),
                    level: current_level,
                    anchor,
                    content: current_lines.join("\n").trim().to_string(),
                });
                current_lines.clear();
            }
            current_heading = heading_text.to_string();
            current_level = level;
        } else if !current_heading.is_empty() {
            current_lines.push(*line);
        }
    }

    if !current_heading.is_empty() {
        let anchor = slugify(&current_heading);
        sections.push(DocSection {
            heading: current_heading,
            level: current_level,
            anchor,
            content: current_lines.join("\n").trim().to_string(),
        });
    }

    sections
}
