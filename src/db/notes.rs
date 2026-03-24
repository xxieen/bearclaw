use anyhow::{Context, Result};

use super::BearDB;
use super::core_data_to_iso;
use super::iso_to_core_data;
use crate::models::{BacklinkNote, Note, NoteFile, NoteSummary};

impl BearDB {
    pub fn read_note_by_id(&self, id: &str) -> Result<Option<Note>> {
        let mut stmt = self.conn().prepare(
            "SELECT n.ZUNIQUEIDENTIFIER, n.ZTITLE, n.ZTEXT, n.ZCREATIONDATE, n.ZMODIFICATIONDATE,
                    n.ZPINNED, n.ZARCHIVED, n.ZTRASHED, n.ZHASFILES, n.ZHASIMAGES, n.ZENCRYPTED
             FROM ZSFNOTE n
             WHERE n.ZUNIQUEIDENTIFIER = ?1
               AND n.ZPERMANENTLYDELETED = 0",
        )?;

        let note = stmt
            .query_row([id], |row| {
                let encrypted: i32 = row.get(10)?;
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i32>(6)?,
                    row.get::<_, i32>(7)?,
                    row.get::<_, i32>(8)?,
                    row.get::<_, i32>(9)?,
                    encrypted,
                ))
            })
            .optional()
            .context("Failed to query note")?;

        match note {
            Some((uid, title, text, created, modified, pinned, archived, trashed, has_files, has_images, encrypted)) => {
                if encrypted != 0 {
                    anyhow::bail!("ENCRYPTED_NOTE");
                }
                let tags = self.get_note_tags_by_uid(&uid)?;
                Ok(Some(Note {
                    id: uid,
                    title: title.unwrap_or_default(),
                    text: text.unwrap_or_default(),
                    tags,
                    created_at: core_data_to_iso(created),
                    modified_at: core_data_to_iso(modified),
                    is_pinned: pinned != 0,
                    is_archived: archived != 0,
                    is_trashed: trashed != 0,
                    has_files: has_files != 0,
                    has_images: has_images != 0,
                }))
            }
            None => Ok(None),
        }
    }

    pub fn read_note_by_title(&self, title: &str) -> Result<Vec<Note>> {
        let mut stmt = self.conn().prepare(
            "SELECT n.ZUNIQUEIDENTIFIER, n.ZTITLE, n.ZTEXT, n.ZCREATIONDATE, n.ZMODIFICATIONDATE,
                    n.ZPINNED, n.ZARCHIVED, n.ZTRASHED, n.ZHASFILES, n.ZHASIMAGES
             FROM ZSFNOTE n
             WHERE n.ZTITLE = ?1
               AND n.ZTRASHED = 0 AND n.ZPERMANENTLYDELETED = 0 AND n.ZENCRYPTED = 0",
        )?;

        let notes = stmt
            .query_map([title], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i32>(6)?,
                    row.get::<_, i32>(7)?,
                    row.get::<_, i32>(8)?,
                    row.get::<_, i32>(9)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for (uid, title, text, created, modified, pinned, archived, trashed, has_files, has_images) in notes {
            let tags = self.get_note_tags_by_uid(&uid)?;
            result.push(Note {
                id: uid,
                title: title.unwrap_or_default(),
                text: text.unwrap_or_default(),
                tags,
                created_at: core_data_to_iso(created),
                modified_at: core_data_to_iso(modified),
                is_pinned: pinned != 0,
                is_archived: archived != 0,
                is_trashed: trashed != 0,
                has_files: has_files != 0,
                has_images: has_images != 0,
            });
        }
        Ok(result)
    }

    pub fn search_notes(
        &self,
        query: &str,
        ocr: bool,
        tag: Option<&str>,
        since: Option<&str>,
        before: Option<&str>,
        limit: u32,
    ) -> Result<Vec<NoteSummary>> {
        let mut sql = String::from(
            "SELECT DISTINCT n.ZUNIQUEIDENTIFIER, n.ZTITLE, n.ZCREATIONDATE, n.ZMODIFICATIONDATE, n.ZPINNED
             FROM ZSFNOTE n",
        );
        let mut conditions = vec![
            "n.ZTRASHED = 0".to_string(),
            "n.ZPERMANENTLYDELETED = 0".to_string(),
            "n.ZENCRYPTED = 0".to_string(),
        ];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if ocr {
            sql.push_str(" LEFT JOIN ZSFNOTEFILE f ON f.ZNOTE = n.Z_PK");
            let pattern = format!("%{query}%");
            conditions.push(format!(
                "(n.ZTEXT LIKE ?{} OR n.ZTITLE LIKE ?{} OR f.ZSEARCHTEXT LIKE ?{})",
                params.len() + 1,
                params.len() + 2,
                params.len() + 3
            ));
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        } else {
            let pattern = format!("%{query}%");
            conditions.push(format!(
                "(n.ZTEXT LIKE ?{} OR n.ZTITLE LIKE ?{})",
                params.len() + 1,
                params.len() + 2
            ));
            params.push(Box::new(pattern.clone()));
            params.push(Box::new(pattern));
        }

        if let Some(tag_name) = tag {
            sql.push_str(
                " JOIN Z_5TAGS zt ON zt.Z_5NOTES = n.Z_PK
                 JOIN ZSFNOTETAG t ON t.Z_PK = zt.Z_13TAGS",
            );
            conditions.push(format!("t.ZTITLE = ?{}", params.len() + 1));
            params.push(Box::new(tag_name.to_string()));
        }

        if let Some(since_date) = since {
            let cd = iso_to_core_data(since_date)?;
            conditions.push(format!("n.ZMODIFICATIONDATE >= ?{}", params.len() + 1));
            params.push(Box::new(cd));
        }

        if let Some(before_date) = before {
            let cd = iso_to_core_data(before_date)?;
            conditions.push(format!("n.ZMODIFICATIONDATE < ?{}", params.len() + 1));
            params.push(Box::new(cd));
        }

        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY n.ZMODIFICATIONDATE DESC");
        sql.push_str(&format!(" LIMIT {limit}"));

        let mut stmt = self.conn().prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, f64>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, i32>(4)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for (uid, title, created, modified, pinned) in rows {
            let tags = self.get_note_tags_by_uid(&uid)?;
            result.push(NoteSummary {
                id: uid,
                title: title.unwrap_or_default(),
                created_at: core_data_to_iso(created),
                modified_at: core_data_to_iso(modified),
                is_pinned: pinned != 0,
                tags,
            });
        }
        Ok(result)
    }

    pub fn get_note_files(&self, note_uid: &str) -> Result<Vec<NoteFile>> {
        let mut stmt = self.conn().prepare(
            "SELECT f.ZFILENAME, f.ZSEARCHTEXT
             FROM ZSFNOTEFILE f
             JOIN ZSFNOTE n ON f.ZNOTE = n.Z_PK
             WHERE n.ZUNIQUEIDENTIFIER = ?1",
        )?;

        let files = stmt
            .query_map([note_uid], |row| {
                Ok(NoteFile {
                    filename: row.get::<_, Option<String>>(0)?.unwrap_or_default(),
                    search_text: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(files)
    }

    fn get_note_tags_by_uid(&self, uid: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn().prepare(
            "SELECT t.ZTITLE FROM ZSFNOTETAG t
             JOIN Z_5TAGS zt ON t.Z_PK = zt.Z_13TAGS
             JOIN ZSFNOTE n ON n.Z_PK = zt.Z_5NOTES
             WHERE n.ZUNIQUEIDENTIFIER = ?1
             ORDER BY t.ZTITLE",
        )?;

        let tags = stmt
            .query_map([uid], |row| row.get::<_, String>(0))?
            .collect::<Result<Vec<_>, _>>()?;
        Ok(tags)
    }

    pub fn get_backlinks(&self, id_or_title: &str) -> Result<Vec<BacklinkNote>> {
        // Try by ID first
        let mut stmt = self.conn().prepare(
            "SELECT DISTINCT n.ZUNIQUEIDENTIFIER, n.ZTITLE
             FROM ZSFNOTEBACKLINK bl
             JOIN ZSFNOTE target ON target.Z_PK = bl.ZLINKINGTO
             JOIN ZSFNOTE n ON n.Z_PK = bl.ZLINKEDBY
             WHERE target.ZUNIQUEIDENTIFIER = ?1
               AND n.ZTRASHED = 0 AND n.ZPERMANENTLYDELETED = 0",
        )?;

        let mut backlinks: Vec<BacklinkNote> = stmt
            .query_map([id_or_title], |row| {
                Ok(BacklinkNote {
                    id: row.get::<_, String>(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        if backlinks.is_empty() {
            // Try by title
            let mut stmt = self.conn().prepare(
                "SELECT DISTINCT n.ZUNIQUEIDENTIFIER, n.ZTITLE
                 FROM ZSFNOTEBACKLINK bl
                 JOIN ZSFNOTE target ON target.Z_PK = bl.ZLINKINGTO
                 JOIN ZSFNOTE n ON n.Z_PK = bl.ZLINKEDBY
                 WHERE target.ZTITLE = ?1
                   AND n.ZTRASHED = 0 AND n.ZPERMANENTLYDELETED = 0",
            )?;

            backlinks = stmt
                .query_map([id_or_title], |row| {
                    Ok(BacklinkNote {
                        id: row.get::<_, String>(0)?,
                        title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    })
                })?
                .collect::<Result<Vec<_>, _>>()?;
        }

        Ok(backlinks)
    }

    pub fn get_section(&self, id_or_title: &str, header: &str) -> Result<Option<String>> {
        let note = self.read_note_by_id(id_or_title)?;
        let note = match note {
            Some(n) => n,
            None => {
                let notes = self.read_note_by_title(id_or_title)?;
                match notes.len() {
                    0 => return Ok(None),
                    1 => notes.into_iter().next().unwrap(),
                    _ => anyhow::bail!("AMBIGUOUS_TITLE"),
                }
            }
        };

        Ok(extract_section(&note.text, header))
    }

    pub fn get_untagged_notes(&self) -> Result<Vec<NoteSummary>> {
        let mut stmt = self.conn().prepare(
            "SELECT n.ZUNIQUEIDENTIFIER, n.ZTITLE, n.ZCREATIONDATE, n.ZMODIFICATIONDATE, n.ZPINNED
             FROM ZSFNOTE n
             WHERE n.ZTRASHED = 0 AND n.ZPERMANENTLYDELETED = 0 AND n.ZENCRYPTED = 0
               AND n.Z_PK NOT IN (SELECT Z_5NOTES FROM Z_5TAGS)
             ORDER BY n.ZMODIFICATIONDATE DESC",
        )?;

        let notes = stmt
            .query_map([], |row| {
                Ok(NoteSummary {
                    id: row.get::<_, String>(0)?,
                    title: row.get::<_, Option<String>>(1)?.unwrap_or_default(),
                    created_at: String::new(),
                    modified_at: String::new(),
                    is_pinned: row.get::<_, i32>(4)? != 0,
                    tags: vec![],
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Fix dates
        let mut result = Vec::new();
        for mut note in notes {
            // Re-read with proper date conversion
            let mut stmt2 = self.conn().prepare(
                "SELECT ZCREATIONDATE, ZMODIFICATIONDATE FROM ZSFNOTE WHERE ZUNIQUEIDENTIFIER = ?1",
            )?;
            if let Ok((created, modified)) = stmt2.query_row([&note.id], |row| {
                Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?))
            }) {
                note.created_at = core_data_to_iso(created);
                note.modified_at = core_data_to_iso(modified);
            }
            result.push(note);
        }
        Ok(result)
    }

    pub fn note_exists(&self, id: &str) -> Result<bool> {
        let count: i32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZUNIQUEIDENTIFIER = ?1 AND ZPERMANENTLYDELETED = 0",
            [id],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn get_note_modified_at(&self, id: &str) -> Result<Option<f64>> {
        let result = self
            .conn()
            .prepare("SELECT ZMODIFICATIONDATE FROM ZSFNOTE WHERE ZUNIQUEIDENTIFIER = ?1")?
            .query_row([id], |row| row.get::<_, f64>(0))
            .optional()?;
        Ok(result)
    }

    pub fn get_all_notes_for_export(
        &self,
        tag: Option<&str>,
        since: Option<&str>,
        before: Option<&str>,
    ) -> Result<Vec<Note>> {
        let mut sql = String::from(
            "SELECT DISTINCT n.ZUNIQUEIDENTIFIER, n.ZTITLE, n.ZTEXT, n.ZCREATIONDATE, n.ZMODIFICATIONDATE,
                    n.ZPINNED, n.ZARCHIVED, n.ZTRASHED, n.ZHASFILES, n.ZHASIMAGES
             FROM ZSFNOTE n",
        );
        let mut conditions = vec![
            "n.ZTRASHED = 0".to_string(),
            "n.ZPERMANENTLYDELETED = 0".to_string(),
            "n.ZENCRYPTED = 0".to_string(),
        ];
        let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = Vec::new();

        if let Some(tag_name) = tag {
            sql.push_str(
                " JOIN Z_5TAGS zt ON zt.Z_5NOTES = n.Z_PK
                 JOIN ZSFNOTETAG t ON t.Z_PK = zt.Z_13TAGS",
            );
            conditions.push(format!("t.ZTITLE = ?{}", params.len() + 1));
            params.push(Box::new(tag_name.to_string()));
        }

        if let Some(since_date) = since {
            let cd = iso_to_core_data(since_date)?;
            conditions.push(format!("n.ZCREATIONDATE >= ?{}", params.len() + 1));
            params.push(Box::new(cd));
        }

        if let Some(before_date) = before {
            let cd = iso_to_core_data(before_date)?;
            conditions.push(format!("n.ZCREATIONDATE < ?{}", params.len() + 1));
            params.push(Box::new(cd));
        }

        sql.push_str(" WHERE ");
        sql.push_str(&conditions.join(" AND "));
        sql.push_str(" ORDER BY n.ZCREATIONDATE DESC");

        let mut stmt = self.conn().prepare(&sql)?;
        let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();
        let rows = stmt
            .query_map(param_refs.as_slice(), |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, f64>(3)?,
                    row.get::<_, f64>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, i32>(6)?,
                    row.get::<_, i32>(7)?,
                    row.get::<_, i32>(8)?,
                    row.get::<_, i32>(9)?,
                ))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        let mut result = Vec::new();
        for (uid, title, text, created, modified, pinned, archived, trashed, has_files, has_images) in rows {
            let tags = self.get_note_tags_by_uid(&uid)?;
            result.push(Note {
                id: uid,
                title: title.unwrap_or_default(),
                text: text.unwrap_or_default(),
                tags,
                created_at: core_data_to_iso(created),
                modified_at: core_data_to_iso(modified),
                is_pinned: pinned != 0,
                is_archived: archived != 0,
                is_trashed: trashed != 0,
                has_files: has_files != 0,
                has_images: has_images != 0,
            });
        }
        Ok(result)
    }
}

fn extract_section(text: &str, header: &str) -> Option<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut start = None;
    let mut header_level = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if let Some(hashes) = trimmed.strip_suffix(|_: char| true).map(|_| trimmed) {
            let level = hashes.chars().take_while(|c| *c == '#').count();
            if level > 0 {
                let h_text = trimmed[level..].trim();
                if start.is_none() && h_text.eq_ignore_ascii_case(header) {
                    start = Some(i + 1);
                    header_level = level;
                } else if start.is_some() && level <= header_level {
                    let content = lines[start.unwrap()..i].join("\n");
                    return Some(content.trim().to_string());
                }
            }
        }
    }

    if let Some(s) = start {
        let content = lines[s..].join("\n");
        return Some(content.trim().to_string());
    }

    None
}

use rusqlite::OptionalExtension;
