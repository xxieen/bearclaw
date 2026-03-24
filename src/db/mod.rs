pub mod notes;
pub mod stats;
pub mod tags;

use anyhow::{Context, Result};
use rusqlite::Connection;
use std::path::PathBuf;

const CORE_DATA_EPOCH_OFFSET: f64 = 978307200.0;

pub fn core_data_to_iso(timestamp: f64) -> String {
    let unix_ts = timestamp + CORE_DATA_EPOCH_OFFSET;
    chrono::DateTime::from_timestamp(unix_ts as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%SZ").to_string())
        .unwrap_or_default()
}

pub fn iso_to_core_data(iso: &str) -> Result<f64> {
    let dt = chrono::NaiveDate::parse_from_str(iso, "%Y-%m-%d")
        .context("Invalid date format, expected YYYY-MM-DD")?;
    let datetime = dt
        .and_hms_opt(0, 0, 0)
        .context("Failed to create datetime")?;
    let unix_ts = datetime.and_utc().timestamp() as f64;
    Ok(unix_ts - CORE_DATA_EPOCH_OFFSET)
}

pub struct BearDB {
    conn: Connection,
}

impl BearDB {
    pub fn open(db_path: &PathBuf) -> Result<Self> {
        let conn = Connection::open_with_flags(
            db_path,
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
        )
        .context("Failed to open Bear database")?;
        Ok(Self { conn })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn default_db_path() -> Option<PathBuf> {
        dirs::home_dir().map(|home| {
            home.join("Library/Group Containers/9K33E3U3T4.net.shinyfrog.bear/Application Data/database.sqlite")
        })
    }

    pub fn check_schema(&self) -> bool {
        let required_tables = [
            "ZSFNOTE",
            "ZSFNOTETAG",
            "Z_5TAGS",
            "ZSFNOTEFILE",
            "ZSFNOTEBACKLINK",
        ];
        for table in &required_tables {
            let exists: bool = self
                .conn
                .prepare(&format!(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='{table}'"
                ))
                .and_then(|mut stmt| stmt.query_row([], |row| row.get::<_, i32>(0)))
                .map(|count| count > 0)
                .unwrap_or(false);
            if !exists {
                return false;
            }
        }
        true
    }
}
