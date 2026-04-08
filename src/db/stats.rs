use anyhow::Result;

use super::BearDB;
use super::core_data_to_iso;
use crate::models::{MonthCount, Stats, TagCount};

impl BearDB {
    pub fn get_stats(&self) -> Result<Stats> {
        let total_notes: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0",
            [],
            |row| row.get(0),
        )?;

        let total_tags: u32 =
            self.conn()
                .query_row("SELECT COUNT(*) FROM ZSFNOTETAG", [], |row| row.get(0))?;

        let archived_count: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZARCHIVED = 1 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0",
            [],
            |row| row.get(0),
        )?;

        let trashed_count: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZTRASHED = 1 AND ZPERMANENTLYDELETED = 0",
            [],
            |row| row.get(0),
        )?;

        let untagged_count: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE
             WHERE ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0
               AND Z_PK NOT IN (SELECT Z_5NOTES FROM Z_5TAGS)",
            [],
            |row| row.get(0),
        )?;

        let pinned_count: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZPINNED = 1 AND ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0",
            [],
            |row| row.get(0),
        )?;

        let with_files_count: u32 = self.conn().query_row(
            "SELECT COUNT(*) FROM ZSFNOTE WHERE ZHASFILES = 1 AND ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0",
            [],
            |row| row.get(0),
        )?;

        // Tag distribution
        let mut stmt = self.conn().prepare(
            "SELECT t.ZTITLE, COUNT(zt.Z_5NOTES) as cnt
             FROM ZSFNOTETAG t
             JOIN Z_5TAGS zt ON t.Z_PK = zt.Z_13TAGS
             JOIN ZSFNOTE n ON n.Z_PK = zt.Z_5NOTES
             WHERE n.ZTRASHED = 0 AND n.ZPERMANENTLYDELETED = 0 AND n.ZENCRYPTED = 0
             GROUP BY t.ZTITLE
             ORDER BY cnt DESC",
        )?;
        let tag_distribution: Vec<TagCount> = stmt
            .query_map([], |row| {
                Ok(TagCount {
                    tag: row.get(0)?,
                    count: row.get(1)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        // Monthly trend (notes created per month)
        let mut stmt = self.conn().prepare(
            "SELECT ZCREATIONDATE FROM ZSFNOTE
             WHERE ZTRASHED = 0 AND ZPERMANENTLYDELETED = 0 AND ZENCRYPTED = 0
             ORDER BY ZCREATIONDATE",
        )?;
        let dates: Vec<f64> = stmt
            .query_map([], |row| row.get::<_, f64>(0))?
            .collect::<Result<Vec<_>, _>>()?;

        let mut month_map: std::collections::BTreeMap<String, u32> =
            std::collections::BTreeMap::new();
        for d in &dates {
            let iso = core_data_to_iso(*d);
            if iso.len() >= 7 {
                let month = iso[..7].to_string();
                *month_map.entry(month).or_insert(0) += 1;
            }
        }
        let monthly_trend: Vec<MonthCount> = month_map
            .into_iter()
            .map(|(month, count)| MonthCount { month, count })
            .collect();

        Ok(Stats {
            total_notes,
            total_tags,
            archived_count,
            trashed_count,
            untagged_count,
            pinned_count,
            with_files_count,
            tag_distribution,
            monthly_trend,
        })
    }
}
