use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bearclaw", about = "CLI tool for Bear notes app", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Pretty-print JSON output
    #[arg(long, global = true)]
    pub pretty: bool,

    /// Custom path to Bear database
    #[arg(long, global = true, env = "BEAR_DB_PATH")]
    pub db_path: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Read a note by ID or title
    Read {
        /// Note ID or title
        id_or_title: String,

        /// Read from trash instead of active notes
        #[arg(long)]
        trashed: bool,
    },

    /// Search notes
    Search {
        /// Search query
        query: String,

        /// Also search OCR text in attachments
        #[arg(long)]
        ocr: bool,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Only notes modified since this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Only notes modified before this date (YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,

        /// Maximum number of results
        #[arg(long, default_value = "50")]
        limit: u32,

        /// Search trash instead of active notes
        #[arg(long)]
        trashed: bool,
    },

    /// Create a new note
    Create {
        /// Note title
        #[arg(long)]
        title: String,

        /// Note body text
        #[arg(long)]
        body: Option<String>,

        /// Read body from file
        #[arg(long)]
        body_file: Option<String>,

        /// Comma-separated tags
        #[arg(long)]
        tags: Option<String>,
    },

    /// Edit (replace) a note's body by ID
    Edit {
        /// Note unique identifier
        id: String,

        /// New body text
        #[arg(long)]
        body: Option<String>,

        /// Read body from file
        #[arg(long)]
        body_file: Option<String>,
    },

    /// Append text to a note by ID
    Append {
        /// Note unique identifier
        id: String,

        /// Text to append
        #[arg(long)]
        text: Option<String>,

        /// Read text from file
        #[arg(long)]
        text_file: Option<String>,

        /// Append under this header
        #[arg(long)]
        header: Option<String>,
    },

    /// Prepend text to a note by ID
    Prepend {
        /// Note unique identifier
        id: String,

        /// Text to prepend
        #[arg(long)]
        text: Option<String>,

        /// Read text from file
        #[arg(long)]
        text_file: Option<String>,
    },

    /// Extract a section from a note by header
    Section {
        /// Note ID or title
        id_or_title: String,

        /// Header text to extract
        #[arg(long)]
        header: String,
    },

    /// Move a note to trash by ID
    Trash {
        /// Note unique identifier
        id: String,
    },

    /// Archive a note by ID
    Archive {
        /// Note unique identifier
        id: String,
    },

    /// Tag operations
    Tag {
        #[command(subcommand)]
        action: TagAction,
    },

    /// List untagged notes
    Untagged {
        /// List untagged notes from trash instead of active notes
        #[arg(long)]
        trashed: bool,
    },

    /// Find notes that link to a given note
    Backlinks {
        /// Note ID or title
        id_or_title: String,
    },

    /// Show notes statistics
    Stats,

    /// Batch operations
    Batch {
        #[command(subcommand)]
        action: BatchAction,
    },

    /// Export notes as Markdown files
    Export {
        /// Output directory
        #[arg(long)]
        output: String,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Only notes created since this date (YYYY-MM-DD)
        #[arg(long)]
        since: Option<String>,

        /// Only notes created before this date (YYYY-MM-DD)
        #[arg(long)]
        before: Option<String>,
    },

    /// Check Bear installation and database status
    Health,
}

#[derive(Subcommand)]
pub enum TagAction {
    /// List all tags as a hierarchical tree
    List {
        /// List tags for trash instead of active notes
        #[arg(long)]
        trashed: bool,
    },

    /// Add tags to a note by ID
    Add {
        /// Note unique identifier
        id: String,

        /// Comma-separated tags to add
        #[arg(long)]
        tags: String,
    },

    /// Rename a tag
    Rename {
        /// Current tag name
        old: String,

        /// New tag name
        new: String,
    },

    /// Delete a tag
    Delete {
        /// Tag name to delete
        name: String,
    },
}

#[derive(Subcommand)]
pub enum BatchAction {
    /// Add tags to multiple notes matching a search
    Tag {
        /// Search filter
        #[arg(long)]
        filter: String,

        /// Comma-separated tags to add
        #[arg(long)]
        tags: String,
    },

    /// Archive multiple notes matching a search
    Archive {
        /// Search filter
        #[arg(long)]
        filter: String,
    },
}
