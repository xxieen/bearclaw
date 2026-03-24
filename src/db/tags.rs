use anyhow::Result;

use super::BearDB;
use crate::models::Tag;

impl BearDB {
    pub fn list_tags(&self) -> Result<Vec<Tag>> {
        let mut stmt = self.conn().prepare(
            "SELECT t.ZTITLE, COUNT(zt.Z_5NOTES) as note_count
             FROM ZSFNOTETAG t
             LEFT JOIN Z_5TAGS zt ON t.Z_PK = zt.Z_13TAGS
             GROUP BY t.ZTITLE
             ORDER BY t.ZTITLE",
        )?;

        let flat_tags: Vec<(String, u32)> = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, u32>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(build_tag_tree(&flat_tags))
    }
}

fn build_tag_tree(flat_tags: &[(String, u32)]) -> Vec<Tag> {
    let mut roots: Vec<Tag> = Vec::new();

    for (name, count) in flat_tags {
        let parts: Vec<&str> = name.split('/').collect();
        insert_tag(&mut roots, &parts, name, *count);
    }

    roots
}

fn insert_tag(tags: &mut Vec<Tag>, parts: &[&str], full_name: &str, count: u32) {
    if parts.is_empty() {
        return;
    }

    let root_name = parts[0];

    // Find or create the root tag
    let existing = tags.iter_mut().find(|t| {
        let t_parts: Vec<&str> = t.name.split('/').collect();
        t_parts.last() == Some(&root_name)
    });

    if parts.len() == 1 {
        // This is a leaf or root tag
        if let Some(tag) = existing {
            tag.note_count = count;
        } else {
            tags.push(Tag {
                name: full_name.to_string(),
                note_count: count,
                children: Vec::new(),
            });
        }
    } else {
        // Need to go deeper
        if let Some(tag) = existing {
            insert_tag(&mut tag.children, &parts[1..], full_name, count);
        } else {
            let mut new_tag = Tag {
                name: root_name.to_string(),
                note_count: 0,
                children: Vec::new(),
            };
            insert_tag(&mut new_tag.children, &parts[1..], full_name, count);
            tags.push(new_tag);
        }
    }
}
