use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub text: String,
    pub tags: Vec<String>,
    pub created_at: String,
    pub modified_at: String,
    pub is_pinned: bool,
    pub is_archived: bool,
    pub is_trashed: bool,
    pub has_files: bool,
    pub has_images: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub modified_at: String,
    pub is_pinned: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Tag {
    pub name: String,
    pub note_count: u32,
    pub children: Vec<Tag>,
}

#[derive(Debug, Serialize, Clone)]
pub struct NoteFile {
    pub filename: String,
    pub search_text: Option<String>,
}

#[derive(Debug, Serialize, Clone)]
pub struct Stats {
    pub total_notes: u32,
    pub total_tags: u32,
    pub archived_count: u32,
    pub trashed_count: u32,
    pub untagged_count: u32,
    pub pinned_count: u32,
    pub with_files_count: u32,
    pub tag_distribution: Vec<TagCount>,
    pub monthly_trend: Vec<MonthCount>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TagCount {
    pub tag: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct MonthCount {
    pub month: String,
    pub count: u32,
}

#[derive(Debug, Serialize, Clone)]
pub struct BacklinkNote {
    pub id: String,
    pub title: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct HealthStatus {
    pub bear_installed: bool,
    pub db_exists: bool,
    pub db_path: String,
    pub note_count: Option<u32>,
    pub tag_count: Option<u32>,
    pub schema_ok: bool,
}
