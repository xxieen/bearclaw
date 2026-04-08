# 🐻🐾 Bearclaw

[中文文档](README_CN.md)

A Rust CLI tool for managing [Bear](https://bear.app) notes on macOS. Designed for AI agent integration with structured JSON output.

## Features

- 📖 **Read & Search** — Full-text search with OCR support for attachments
- ✏️ **Create & Edit** — Create, replace, append, prepend note content
- 🏷️ **Tag Management** — List tags as tree, add/rename/delete tags, find untagged notes
- 🔗 **Backlinks** — Find notes that link to a given note
- 📊 **Statistics** — Note counts, tag distribution, monthly trends
- 📦 **Batch Operations** — Bulk tag and archive
- 📤 **Export** — Export notes as Markdown with YAML frontmatter
- 🔒 **Safe** — Reads from SQLite directly, writes through Bear's official API
- 🤖 **Agent-friendly** — JSON output, designed for AI tool integration

## Installation

> **Important**: The CLI tool must be installed before using any AI agent integration.

### Pre-built binary (recommended)

Download the latest release for your Mac:

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/xxieen/bearclaw/releases/latest/download/bearclaw-aarch64-apple-darwin.tar.gz | tar xz
sudo mv bearclaw /usr/local/bin/

# Intel Mac
curl -L https://github.com/xxieen/bearclaw/releases/latest/download/bearclaw-x86_64-apple-darwin.tar.gz | tar xz
sudo mv bearclaw /usr/local/bin/
```

### From crates.io

```bash
cargo install bearclaw
```

### From source

```bash
git clone https://github.com/xxieen/bearclaw.git
cd bearclaw
cargo install --path .
```

### Verify installation

```bash
bearclaw health
```

### Requirements

- macOS (Bear is macOS/iOS only)
- [Bear](https://bear.app) installed and running (for write operations)

## Quick Start

```bash
# Check everything is working
bearclaw health

# Search notes
bearclaw search "rust programming"

# Read a specific note
bearclaw read <note-id>

# Create a note
bearclaw create --title "My Note" --body "Hello world" --tags "test,demo"

# View statistics
bearclaw stats
```

## Commands

### Note Operations

| Command                                  | Description                                                       |
| ---------------------------------------- | ----------------------------------------------------------------- |
| `bearclaw read <id-or-title>`            | Read a note's full content; use `--trashed` to read from trash    |
| `bearclaw search <query>`                | Search notes (`--ocr`, `--tag`, `--since`, `--before`, `--limit`, `--trashed`) |
| `bearclaw create --title "x" --body "y"` | Create a new note (`--tags`, `--body-file`)                       |
| `bearclaw edit <id> --body "x"`          | Replace note body (`--body-file`)                                 |
| `bearclaw append <id> --text "x"`        | Append text (`--header`, `--text-file`)                           |
| `bearclaw prepend <id> --text "x"`       | Prepend text (`--text-file`)                                      |
| `bearclaw section <id> --header "x"`     | Extract a section by header                                       |
| `bearclaw trash <id>`                    | Move note to trash                                                |
| `bearclaw archive <id>`                  | Archive a note                                                    |

### Tag Operations

| Command                              | Description                        |
| ------------------------------------ | ---------------------------------- |
| `bearclaw tag list`                  | List all tags as hierarchical tree; use `--trashed` for trash |
| `bearclaw tag add <id> --tags "a,b"` | Add tags to a note                 |
| `bearclaw tag rename <old> <new>`    | Rename a tag                       |
| `bearclaw tag delete <name>`         | Delete a tag                       |
| `bearclaw untagged`                  | List notes without tags; use `--trashed` for trash |

### Analysis & Links

| Command                            | Description                     |
| ---------------------------------- | ------------------------------- |
| `bearclaw backlinks <id-or-title>` | Find notes linking to this note |
| `bearclaw stats`                   | Show statistics                 |

### Batch & Export

| Command                                        | Description                                         |
| ---------------------------------------------- | --------------------------------------------------- |
| `bearclaw batch tag --filter "q" --tags "a,b"` | Bulk add tags                                       |
| `bearclaw batch archive --filter "q"`          | Bulk archive                                        |
| `bearclaw export --output ./dir/`              | Export as Markdown (`--tag`, `--since`, `--before`) |

### Diagnostics

| Command           | Description                                 |
| ----------------- | ------------------------------------------- |
| `bearclaw health` | Check Bear installation and database status |

### Global Options

| Option             | Description                                      |
| ------------------ | ------------------------------------------------ |
| `--pretty`         | Pretty-print JSON output                         |
| `--db-path <path>` | Custom database path (or `BEAR_DB_PATH` env var) |

## JSON Output

All commands return structured JSON:

```json
// Success
{"ok": true, "data": {...}, "count": 10}

// Error
{"ok": false, "error": "Note not found", "code": "NOT_FOUND"}
```

## AI Agent Integration

Bearclaw is designed to work with any AI coding agent. Below are setup guides for popular agents.

> **Prerequisite**: Install the `bearclaw` CLI first (see [Installation](#installation) above).

---

### Amp

Install the bundled skill directly from this repository:

```bash
amp skill add xxieen/bearclaw/bear
```

The skill will be loaded automatically when you ask about Bear notes. First-time use will check if `bearclaw` is installed.

---

### Claude Code

Copy the `.claude/skills/bear/` directory from this repository into your project:

```bash
# Clone and copy the skill
git clone https://github.com/xxieen/bearclaw.git /tmp/bearclaw
cp -r /tmp/bearclaw/.claude/skills/bear .claude/skills/
rm -rf /tmp/bearclaw
```

Or manually create `.claude/skills/bear/SKILL.md` in your project. The skill will be available as `/bear` in Claude Code.

---

## Architecture

- **Reads**: Direct SQLite access (fast, no Bear app needed)
- **Writes**: Bear's official `x-callback-url` API (safe, requires Bear running)
- **Output**: JSON to stdout for agent consumption

## License

MIT
