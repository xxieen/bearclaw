use serde::Serialize;
use std::fs;
use std::path::Path;

use crate::db::BearDB;
use crate::output::{self, Response};

#[derive(Debug, Serialize)]
struct ExportResult {
    exported: u32,
    output_dir: String,
}

pub fn export_notes(db: &BearDB, output_dir: &str, tag: Option<&str>, since: Option<&str>, before: Option<&str>, pretty: bool) {
    let notes = match db.get_all_notes_for_export(tag, since, before) {
        Ok(n) => n,
        Err(e) => {
            output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    let out_path = Path::new(output_dir);
    if let Err(e) = fs::create_dir_all(out_path) {
        output::print_json(&Response::<()>::error("IO_ERROR", &e.to_string()), pretty);
        return;
    }

    let mut exported = 0u32;
    for note in &notes {
        let sanitized_title = sanitize_filename(&note.title);
        let short_id = if note.id.len() > 8 { &note.id[..8] } else { &note.id };
        let filename = if sanitized_title.is_empty() {
            format!("untitled--{short_id}.md")
        } else {
            format!("{sanitized_title}--{short_id}.md")
        };

        let file_path = out_path.join(&filename);

        let mut content = String::new();

        // YAML frontmatter
        content.push_str("---\n");
        if !note.tags.is_empty() {
            let tags_str = note
                .tags
                .iter()
                .map(|t| format!("\"{t}\""))
                .collect::<Vec<_>>()
                .join(", ");
            content.push_str(&format!("tags: [{tags_str}]\n"));
        }
        content.push_str(&format!("created: {}\n", note.created_at));
        content.push_str(&format!("modified: {}\n", note.modified_at));
        content.push_str(&format!("bear_id: {}\n", note.id));
        content.push_str("---\n\n");

        content.push_str(&note.text);

        match fs::write(&file_path, &content) {
            Ok(()) => exported += 1,
            Err(e) => {
                eprintln!("Failed to write {}: {e}", file_path.display());
            }
        }
    }

    output::print_json(
        &Response::success(ExportResult {
            exported,
            output_dir: output_dir.to_string(),
        }),
        pretty,
    );
}

fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            '\0' => '_',
            _ => c,
        })
        .collect::<String>()
        .trim()
        .to_string()
}
