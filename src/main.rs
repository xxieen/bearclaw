use std::path::PathBuf;
use std::process;

use clap::Parser;

use bearclaw::cli::args::{BatchAction, Cli, Commands, TagAction};
use bearclaw::commands::{batch, export, note, stats, tag};
use bearclaw::db::BearDB;
use bearclaw::models::HealthStatus;
use bearclaw::output::{self, Response};

fn main() {
    let cli = Cli::parse();
    let pretty = cli.pretty;

    let db_path = resolve_db_path(&cli);

    if let Commands::Health = cli.command {
        run_health(&db_path, pretty);
        return;
    }

    let db = match open_db(&db_path, pretty) {
        Some(db) => db,
        None => process::exit(1),
    };

    match cli.command {
        Commands::Read {
            id_or_title,
            trashed,
        } => {
            note::read_note(&db, &id_or_title, trashed, pretty);
        }
        Commands::Search {
            query,
            ocr,
            tag: tag_filter,
            since,
            before,
            limit,
            trashed,
        } => {
            note::search_notes(
                &db,
                &query,
                ocr,
                tag_filter.as_deref(),
                since.as_deref(),
                before.as_deref(),
                limit,
                trashed,
                pretty,
            );
        }
        Commands::Create {
            title,
            body,
            body_file,
            tags,
        } => {
            note::create_note(
                &title,
                body.as_deref(),
                body_file.as_deref(),
                tags.as_deref(),
                pretty,
            );
        }
        Commands::Edit {
            id,
            body,
            body_file,
        } => {
            note::edit_note(&db, &id, body.as_deref(), body_file.as_deref(), pretty);
        }
        Commands::Append {
            id,
            text,
            text_file,
            header,
        } => {
            note::append_text(
                &db,
                &id,
                text.as_deref(),
                text_file.as_deref(),
                header.as_deref(),
                pretty,
            );
        }
        Commands::Prepend {
            id,
            text,
            text_file,
        } => {
            note::prepend_text(&db, &id, text.as_deref(), text_file.as_deref(), pretty);
        }
        Commands::Section {
            id_or_title,
            header,
        } => {
            note::section(&db, &id_or_title, &header, pretty);
        }
        Commands::Trash { id } => {
            note::trash_note(&id, pretty);
        }
        Commands::Archive { id } => {
            note::archive_note(&id, pretty);
        }
        Commands::Tag { action } => match action {
            TagAction::List { trashed } => tag::list_tags(&db, trashed, pretty),
            TagAction::Add { id, tags } => tag::add_tag(&db, &id, &tags, pretty),
            TagAction::Rename { old, new } => tag::rename_tag(&old, &new, pretty),
            TagAction::Delete { name } => tag::delete_tag(&name, pretty),
        },
        Commands::Untagged { trashed } => {
            tag::untagged(&db, trashed, pretty);
        }
        Commands::Backlinks { id_or_title } => {
            note::backlinks(&db, &id_or_title, pretty);
        }
        Commands::Stats => {
            stats::show_stats(&db, pretty);
        }
        Commands::Batch { action } => match action {
            BatchAction::Tag { filter, tags } => batch::batch_tag(&db, &filter, &tags, pretty),
            BatchAction::Archive { filter } => batch::batch_archive(&db, &filter, pretty),
        },
        Commands::Export {
            output: out_dir,
            tag: tag_filter,
            since,
            before,
        } => {
            export::export_notes(
                &db,
                &out_dir,
                tag_filter.as_deref(),
                since.as_deref(),
                before.as_deref(),
                pretty,
            );
        }
        Commands::Health => unreachable!(),
    }
}

fn resolve_db_path(cli: &Cli) -> PathBuf {
    if let Some(ref path) = cli.db_path {
        PathBuf::from(path)
    } else {
        BearDB::default_db_path().unwrap_or_default()
    }
}

fn open_db(db_path: &PathBuf, pretty: bool) -> Option<BearDB> {
    if !db_path.exists() {
        output::print_json(
            &Response::<()>::error(
                "DB_NOT_FOUND",
                &format!("Database not found at: {}", db_path.display()),
            ),
            pretty,
        );
        return None;
    }
    match BearDB::open(db_path) {
        Ok(db) => Some(db),
        Err(e) => {
            output::print_json(&Response::<()>::error("DB_ERROR", &e.to_string()), pretty);
            None
        }
    }
}

fn run_health(db_path: &PathBuf, pretty: bool) {
    let bear_installed = std::process::Command::new("mdfind")
        .args(["kMDItemCFBundleIdentifier == 'net.shinyfrog.bear'"])
        .output()
        .map(|o| !o.stdout.is_empty())
        .unwrap_or(false);

    let db_exists = db_path.exists();

    let mut status = HealthStatus {
        bear_installed,
        db_exists,
        db_path: db_path.display().to_string(),
        note_count: None,
        tag_count: None,
        schema_ok: false,
    };

    if db_exists && let Ok(db) = BearDB::open(db_path) {
        status.schema_ok = db.check_schema();
        if status.schema_ok
            && let Ok(stats) = db.get_stats()
        {
            status.note_count = Some(stats.total_notes);
            status.tag_count = Some(stats.total_tags);
        }
    }

    output::print_json(&Response::success(status), pretty);
}
