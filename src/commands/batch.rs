use serde::Serialize;

use crate::api;
use crate::db::BearDB;
use crate::output::{self, Response};

#[derive(Debug, Serialize)]
struct BatchResult {
    succeeded: Vec<String>,
    failed: Vec<BatchFailure>,
}

#[derive(Debug, Serialize)]
struct BatchFailure {
    id: String,
    error: String,
}

pub fn batch_tag(db: &BearDB, filter: &str, tags: &str, pretty: bool) {
    let notes = match db.search_notes(filter, false, None, None, None, 1000) {
        Ok(n) => n,
        Err(e) => {
            output::print_json(&Response::<()>::error("SEARCH_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    if notes.is_empty() {
        output::print_json(&Response::<()>::error("NOT_FOUND", "No notes match the filter"), pretty);
        return;
    }

    let mut result = BatchResult {
        succeeded: Vec::new(),
        failed: Vec::new(),
    };

    for note in &notes {
        match api::add_tag(&note.id, tags, db) {
            Ok(true) => result.succeeded.push(note.id.clone()),
            Ok(false) => result.failed.push(BatchFailure {
                id: note.id.clone(),
                error: "VERIFY_TIMEOUT".to_string(),
            }),
            Err(e) => result.failed.push(BatchFailure {
                id: note.id.clone(),
                error: e.to_string(),
            }),
        }
    }

    let total = notes.len();
    if result.failed.is_empty() {
        output::print_json(&Response::success_with_count(result, total), pretty);
    } else {
        // Partial failure
        let resp = Response {
            ok: false,
            data: Some(result),
            count: Some(total),
            error: Some("Some operations failed".to_string()),
            code: Some("PARTIAL_FAILURE".to_string()),
        };
        output::print_json(&resp, pretty);
    }
}

pub fn batch_archive(db: &BearDB, filter: &str, pretty: bool) {
    let notes = match db.search_notes(filter, false, None, None, None, 1000) {
        Ok(n) => n,
        Err(e) => {
            output::print_json(&Response::<()>::error("SEARCH_ERROR", &e.to_string()), pretty);
            return;
        }
    };

    if notes.is_empty() {
        output::print_json(&Response::<()>::error("NOT_FOUND", "No notes match the filter"), pretty);
        return;
    }

    let mut result = BatchResult {
        succeeded: Vec::new(),
        failed: Vec::new(),
    };

    for note in &notes {
        match api::archive_note(&note.id) {
            Ok(()) => result.succeeded.push(note.id.clone()),
            Err(e) => result.failed.push(BatchFailure {
                id: note.id.clone(),
                error: e.to_string(),
            }),
        }
    }

    let total = notes.len();
    if result.failed.is_empty() {
        output::print_json(&Response::success_with_count(result, total), pretty);
    } else {
        let resp = Response {
            ok: false,
            data: Some(result),
            count: Some(total),
            error: Some("Some operations failed".to_string()),
            code: Some("PARTIAL_FAILURE".to_string()),
        };
        output::print_json(&resp, pretty);
    }
}
