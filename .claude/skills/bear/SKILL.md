---
name: bear
description: "Manages Bear notes via bearclaw. Use when asked to read, search, create, edit, organize, tag, archive, or export Bear notes."
---

# Bear Notes Management

Manage Bear notes using the `bearclaw` command-line tool. All commands output JSON.

## Setup (run once)

Before first use, run `scripts/ensure-installed.sh` to check and install `bearclaw` if needed.
Then run `bearclaw health` to verify Bear database access.

## Available Commands

### Reading Notes

```bash
# Read a note by ID or title
bearclaw read <id-or-title>

# Search notes (supports OCR search in attachments)
bearclaw search <query> [--ocr] [--tag <tag>] [--since YYYY-MM-DD] [--before YYYY-MM-DD] [--limit N]

# Extract a specific section by header
bearclaw section <id-or-title> --header "Section Name"

# Find notes linking to a given note
bearclaw backlinks <id-or-title>

# List untagged notes
bearclaw untagged

# Show statistics (total notes, tag distribution, monthly trends)
bearclaw stats
```

### Writing Notes

All write commands use note ID only (not title) to avoid ambiguity. Bear app must be running.

```bash
# Create a new note
bearclaw create --title "Title" --body "Content with\nnewlines" --tags "tag1,tag2"

# Replace note body (keeps title)
bearclaw edit <id> --body "New content"

# Append text (optionally under a specific header)
bearclaw append <id> --text "Text to append" [--header "Section Name"]

# Prepend text
bearclaw prepend <id> --text "Text to prepend"

# Move to trash
bearclaw trash <id>

# Archive
bearclaw archive <id>
```

For large content, use `--body-file <path>` or `--text-file <path>` instead of inline text. Use `\n` for newlines in inline text.

### Tag Management

```bash
# List all tags as hierarchical tree with note counts
bearclaw tag list

# Add tags to a note
bearclaw tag add <id> --tags "tag1,tag2"

# Rename a tag across all notes
bearclaw tag rename <old-name> <new-name>

# Delete a tag from all notes
bearclaw tag delete <name>
```

### Batch Operations

```bash
# Add tags to all notes matching a search
bearclaw batch tag --filter "search query" --tags "tag1,tag2"

# Archive all notes matching a search
bearclaw batch archive --filter "search query"
```

### Export

```bash
# Export notes as Markdown files with YAML frontmatter
bearclaw export --output ./notes/ [--tag <tag>] [--since YYYY-MM-DD] [--before YYYY-MM-DD]
```

### Diagnostics

```bash
bearclaw health   # Check Bear installation, database, schema status
```

## JSON Response Format

Success:
```json
{"ok": true, "data": ..., "count": 10}
```

Error:
```json
{"ok": false, "error": "message", "code": "ERROR_CODE"}
```

Error codes: `NOT_FOUND`, `AMBIGUOUS_TITLE`, `ENCRYPTED_NOTE`, `VERIFY_TIMEOUT`, `PAYLOAD_TOO_LARGE`, `DB_NOT_FOUND`, `BEAR_NOT_INSTALLED`

## Common Workflows

### Organize untagged notes
1. `bearclaw untagged` — get all untagged notes
2. `bearclaw read <id>` — read each note's content
3. `bearclaw tag list` — see existing tag structure
4. `bearclaw tag add <id> --tags "appropriate-tag"` — assign tags

### Weekly review
1. `bearclaw search "" --since YYYY-MM-DD` — find notes from this week
2. Read and summarize each note
3. Tag, archive, or update as needed

### Knowledge synthesis
1. `bearclaw search "topic"` — find all notes on a topic
2. `bearclaw read <id>` for each — gather content
3. `bearclaw create --title "Summary" --body "..." --tags "summary"` — create synthesis note

### Batch cleanup
1. `bearclaw search "old-query"` — find notes to clean up
2. `bearclaw batch tag --filter "query" --tags "archived"` — tag them
3. `bearclaw batch archive --filter "query"` — archive them

## Important Notes

- Always use note **ID** (not title) for write operations
- Use `bearclaw search` or `bearclaw read` first to get the note ID
- Convert user's natural language dates to ISO format (YYYY-MM-DD) before passing to `--since`/`--before`
- Use `\n` for newlines in `--body` and `--text` parameters
- For large content, prefer `--body-file` or `--text-file`
- Bear app must be running for write operations to work
