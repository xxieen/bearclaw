use crate::db::BearDB;
use crate::output::{self, Response};

pub fn show_stats(db: &BearDB, pretty: bool) {
    match db.get_stats() {
        Ok(stats) => output::print_json(&Response::success(stats), pretty),
        Err(e) => output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty),
    }
}
