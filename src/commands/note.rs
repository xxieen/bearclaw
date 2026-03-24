use anyhow::Result;
use std::io::Read as _;

use crate::api;
use crate::db::BearDB;
use crate::models::Note;
use crate::output::{self, Response};

pub fn read_note(db: &BearDB, id_or_title: &str, pretty: bool) {
    match do_read(db, id_or_title) {
        Ok(Some(note)) => output::print_json(&Response::success(note), pretty),
        Ok(None) => output::print_json(&Response::<()>::error("NOT_FOUND", "Note not found"), pretty),
        Err(e) => {
            let msg = e.to_string();
            if msg == "ENCRYPTED_NOTE" {
                output::print_json(
                    &Response::<()>::error("ENCRYPTED_NOTE", "Note is encrypted and cannot be read"),
                    pretty,
                );
            } else {
                output::print_json(&Response::<()>::error("DB_ERROR", &msg), pretty);
            }
        }
    }
}

fn do_read(db: &BearDB, id_or_title: &str) -> Result<Option<Note>> {
    // Try by ID first
    if let Some(note) = db.read_note_by_id(id_or_title)? {
        return Ok(Some(note));
    }
    // Try by title
    let notes = db.read_note_by_title(id_or_title)?;
    match notes.len() {
        0 => Ok(None),
        1 => Ok(Some(notes.into_iter().next().unwrap())),
        _ => anyhow::bail!("AMBIGUOUS_TITLE"),
    }
}

pub fn search_notes(
    db: &BearDB,
    query: &str,
    ocr: bool,
    tag: Option<&str>,
    since: Option<&str>,
    before: Option<&str>,
    limit: u32,
    pretty: bool,
) {
    match db.search_notes(query, ocr, tag, since, before, limit) {
        Ok(notes) => {
            let count = notes.len();
            output::print_json(&Response::success_with_count(notes, count), pretty);
        }
        Err(e) => output::print_json(&Response::<()>::error("SEARCH_ERROR", &e.to_string()), pretty),
    }
}

pub fn create_note(title: &str, body: Option<&str>, body_file: Option<&str>, tags: Option<&str>, pretty: bool) {
    let body_content = match resolve_text(body, body_file) {
        Ok(t) => t,
        Err(e) => {
            output::print_json(&Response::<()>::error("INPUT_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    let body_str = body_content.as_deref().unwrap_or("");

    match api::create_note(title, body_str, tags, false) {
        Ok(()) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Err(e) => output::print_json(&Response::<()>::error("CREATE_ERROR", &e.to_string()), pretty),
    }
}

pub fn edit_note(db: &BearDB, id: &str, body: Option<&str>, body_file: Option<&str>, pretty: bool) {
    let body_content = match resolve_text(body, body_file) {
        Ok(Some(t)) => t,
        Ok(None) => {
            output::print_json(
                &Response::<()>::error("INPUT_ERROR", "Either --body or --body-file is required"),
                pretty,
            );
            return;
        }
        Err(e) => {
            output::print_json(&Response::<()>::error("INPUT_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    match api::edit_note(id, &body_content, db) {
        Ok(true) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Ok(false) => output::print_json(
            &Response::<()>::error("VERIFY_TIMEOUT", "Write could not be verified"),
            pretty,
        ),
        Err(e) => output::print_json(&Response::<()>::error("EDIT_ERROR", &e.to_string()), pretty),
    }
}

pub fn append_text(
    db: &BearDB,
    id: &str,
    text: Option<&str>,
    text_file: Option<&str>,
    header: Option<&str>,
    pretty: bool,
) {
    let content = match resolve_text(text, text_file) {
        Ok(Some(t)) => t,
        Ok(None) => {
            output::print_json(
                &Response::<()>::error("INPUT_ERROR", "Either --text or --text-file is required"),
                pretty,
            );
            return;
        }
        Err(e) => {
            output::print_json(&Response::<()>::error("INPUT_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    match api::add_text(id, &content, "append", header, db) {
        Ok(true) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Ok(false) => output::print_json(
            &Response::<()>::error("VERIFY_TIMEOUT", "Write could not be verified"),
            pretty,
        ),
        Err(e) => output::print_json(&Response::<()>::error("APPEND_ERROR", &e.to_string()), pretty),
    }
}

pub fn prepend_text(db: &BearDB, id: &str, text: Option<&str>, text_file: Option<&str>, pretty: bool) {
    let content = match resolve_text(text, text_file) {
        Ok(Some(t)) => t,
        Ok(None) => {
            output::print_json(
                &Response::<()>::error("INPUT_ERROR", "Either --text or --text-file is required"),
                pretty,
            );
            return;
        }
        Err(e) => {
            output::print_json(&Response::<()>::error("INPUT_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    match api::add_text(id, &content, "prepend", None, db) {
        Ok(true) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Ok(false) => output::print_json(
            &Response::<()>::error("VERIFY_TIMEOUT", "Write could not be verified"),
            pretty,
        ),
        Err(e) => output::print_json(&Response::<()>::error("PREPEND_ERROR", &e.to_string()), pretty),
    }
}

pub fn section(db: &BearDB, id_or_title: &str, header: &str, pretty: bool) {
    match db.get_section(id_or_title, header) {
        Ok(Some(content)) => output::print_json(&Response::success(content), pretty),
        Ok(None) => output::print_json(
            &Response::<()>::error("NOT_FOUND", "Section not found"),
            pretty,
        ),
        Err(e) => {
            let msg = e.to_string();
            if msg == "AMBIGUOUS_TITLE" {
                output::print_json(
                    &Response::<()>::error("AMBIGUOUS_TITLE", "Multiple notes match this title"),
                    pretty,
                );
            } else {
                output::print_json(&Response::<()>::error("DB_ERROR", &msg), pretty);
            }
        }
    }
}

pub fn trash_note(id: &str, pretty: bool) {
    match api::trash_note(id) {
        Ok(()) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Err(e) => output::print_json(&Response::<()>::error("TRASH_ERROR", &e.to_string()), pretty),
    }
}

pub fn archive_note(id: &str, pretty: bool) {
    match api::archive_note(id) {
        Ok(()) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Err(e) => output::print_json(&Response::<()>::error("ARCHIVE_ERROR", &e.to_string()), pretty),
    }
}

pub fn backlinks(db: &BearDB, id_or_title: &str, pretty: bool) {
    match db.get_backlinks(id_or_title) {
        Ok(links) => {
            let count = links.len();
            output::print_json(&Response::success_with_count(links, count), pretty);
        }
        Err(e) => output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty),
    }
}

fn unescape(s: &str) -> String {
    s.replace("\\n", "\n").replace("\\t", "\t")
}

fn resolve_text(inline: Option<&str>, file_path: Option<&str>) -> Result<Option<String>> {
    if let Some(text) = inline {
        return Ok(Some(unescape(text)));
    }
    if let Some(path) = file_path {
        if path == "-" {
            let mut buf = String::new();
            std::io::stdin().read_to_string(&mut buf)?;
            return Ok(Some(buf));
        }
        let content = std::fs::read_to_string(path)?;
        return Ok(Some(content));
    }
    // Check if stdin has data (non-tty)
    if !atty_is_terminal() {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        if !buf.is_empty() {
            return Ok(Some(buf));
        }
    }
    Ok(None)
}

fn atty_is_terminal() -> bool {
    std::io::IsTerminal::is_terminal(&std::io::stdin())
}
