use crate::api;
use crate::db::{BearDB, NoteLocation};
use crate::output::{self, Response};

fn note_location(trashed: bool) -> NoteLocation {
    if trashed {
        NoteLocation::Trashed
    } else {
        NoteLocation::Active
    }
}

pub fn list_tags(db: &BearDB, trashed: bool, pretty: bool) {
    match db.list_tags(note_location(trashed)) {
        Ok(tags) => {
            let count = tags.len();
            output::print_json(&Response::success_with_count(tags, count), pretty);
        }
        Err(e) => output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty),
    }
}

pub fn add_tag(db: &BearDB, id: &str, tags: &str, pretty: bool) {
    match api::add_tag(id, tags, db) {
        Ok(true) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Ok(false) => output::print_json(
            &Response::<()>::error("VERIFY_TIMEOUT", "Tag addition could not be verified"),
            pretty,
        ),
        Err(e) => output::print_json(&Response::<()>::error("TAG_ERROR", &e.to_string()), pretty),
    }
}

pub fn rename_tag(old: &str, new: &str, pretty: bool) {
    match api::rename_tag(old, new) {
        Ok(()) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Err(e) => output::print_json(
            &Response::<()>::error("RENAME_ERROR", &e.to_string()),
            pretty,
        ),
    }
}

pub fn delete_tag(name: &str, pretty: bool) {
    match api::delete_tag(name) {
        Ok(()) => output::print_json(&Response::<()>::ok_empty(), pretty),
        Err(e) => output::print_json(
            &Response::<()>::error("DELETE_ERROR", &e.to_string()),
            pretty,
        ),
    }
}

pub fn untagged(db: &BearDB, trashed: bool, pretty: bool) {
    match db.get_untagged_notes(note_location(trashed)) {
        Ok(notes) => {
            let count = notes.len();
            output::print_json(&Response::success_with_count(notes, count), pretty);
        }
        Err(e) => output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty),
    }
}
