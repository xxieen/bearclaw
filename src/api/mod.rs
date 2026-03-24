use anyhow::{Context, Result};
use std::process::Command;
use std::thread;
use std::time::Duration;

use crate::db::BearDB;

const BASE_URL: &str = "bear://x-callback-url";

fn build_url(action: &str, params: &[(&str, &str)]) -> String {
    let mut url = format!("{BASE_URL}/{action}?show_window=no");
    for (key, value) in params {
        url.push('&');
        url.push_str(key);
        url.push('=');
        url.push_str(&urlencoding::encode(value));
    }
    url
}

fn open_url(url: &str) -> Result<()> {
    Command::new("open")
        .arg("-g") // open in background, don't bring Bear to foreground
        .arg(url)
        .output()
        .context("Failed to execute 'open' command")?;
    Ok(())
}

fn verify_write(db: &BearDB, note_id: &str, original_modified: Option<f64>) -> Result<bool> {
    let delays = [500, 1000, 1500, 2000, 3000];
    for delay in &delays {
        thread::sleep(Duration::from_millis(*delay));
        match original_modified {
            Some(orig) => {
                if let Ok(Some(new_mod)) = db.get_note_modified_at(note_id)
                    && new_mod > orig
                {
                    return Ok(true);
                }
            }
            None => {
                // For new notes, check if note exists now
                if let Ok(true) = db.note_exists(note_id) {
                    return Ok(true);
                }
            }
        }
    }
    Ok(false)
}

pub fn create_note(
    title: &str,
    body: &str,
    tags: Option<&str>,
    open_note: bool,
) -> Result<()> {
    let mut params = vec![("title", title), ("text", body)];
    if !open_note {
        params.push(("open_note", "no"));
    }
    if let Some(t) = tags {
        params.push(("tags", t));
    }
    let url = build_url("create", &params);
    open_url(&url)
}

pub fn edit_note(id: &str, body: &str, db: &BearDB) -> Result<bool> {
    let orig = db.get_note_modified_at(id)?;
    let params = vec![
        ("id", id),
        ("text", body),
        ("mode", "replace"),
        ("open_note", "no"),
    ];
    let url = build_url("add-text", &params);
    open_url(&url)?;
    verify_write(db, id, orig)
}

pub fn add_text(
    id: &str,
    text: &str,
    mode: &str,
    header: Option<&str>,
    db: &BearDB,
) -> Result<bool> {
    let orig = db.get_note_modified_at(id)?;
    let mut params = vec![("id", id), ("text", text), ("mode", mode), ("open_note", "no")];
    if let Some(h) = header {
        params.push(("header", h));
    }
    let url = build_url("add-text", &params);
    open_url(&url)?;
    verify_write(db, id, orig)
}

pub fn trash_note(id: &str) -> Result<()> {
    let url = build_url("trash", &[("id", id), ("show_window", "no")]);
    open_url(&url)
}

pub fn archive_note(id: &str) -> Result<()> {
    let url = build_url("archive", &[("id", id), ("show_window", "no")]);
    open_url(&url)
}

pub fn add_tag(id: &str, tags: &str, db: &BearDB) -> Result<bool> {
    let orig = db.get_note_modified_at(id)?;
    let params = vec![("id", id), ("tags", tags), ("open_note", "no")];
    let url = build_url("add-text", &params);
    open_url(&url)?;
    verify_write(db, id, orig)
}

pub fn rename_tag(old_name: &str, new_name: &str) -> Result<()> {
    let url = build_url(
        "rename-tag",
        &[("name", old_name), ("new_name", new_name), ("show_window", "no")],
    );
    open_url(&url)
}

pub fn delete_tag(name: &str) -> Result<()> {
    let url = build_url("delete-tag", &[("name", name), ("show_window", "no")]);
    open_url(&url)
}

pub fn add_file(id: &str, file_data_base64: &str, filename: &str) -> Result<()> {
    let params = vec![
        ("id", id),
        ("file", file_data_base64),
        ("filename", filename),
        ("open_note", "no"),
    ];
    let url = build_url("add-file", &params);
    open_url(&url)
}
