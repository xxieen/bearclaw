# 🐻🐾 Bearclaw

一个用 Rust 编写的 macOS 命令行工具，用于管理 [Bear](https://bear.app) 笔记。输出结构化 JSON，专为 AI Agent 集成而设计。

## 功能特性

- 📖 **阅读与搜索** — 全文搜索，支持附件中图片和 PDF 的 OCR 文字搜索
- ✏️ **创建与编辑** — 创建、替换、追加、插入笔记内容
- 🏷️ **标签管理** — 树形展示标签、添加/重命名/删除标签、查找未打标签的笔记
- 🔗 **反向链接** — 查找链接到指定笔记的其他笔记
- 📊 **统计分析** — 笔记总数、标签分布、月度趋势
- 📦 **批量操作** — 批量打标签、批量归档
- 📤 **导出** — 导出为 Markdown 文件，包含 YAML frontmatter
- 🔒 **安全** — 通过 SQLite 只读数据库读取，通过 Bear 官方 API 写入
- 🤖 **AI 友好** — JSON 输出，专为 AI 工具集成设计

## 安装

> **重要**：在使用任何 AI Agent 集成之前，必须先安装 CLI 工具。

### 预编译二进制文件（推荐）

下载适合你 Mac 的最新版本：

```bash
# Apple Silicon (M1/M2/M3/M4)
curl -L https://github.com/xxieen/bearclaw/releases/latest/download/bearclaw-aarch64-apple-darwin.tar.gz | tar xz
sudo mv bearclaw /usr/local/bin/

# Intel Mac
curl -L https://github.com/xxieen/bearclaw/releases/latest/download/bearclaw-x86_64-apple-darwin.tar.gz | tar xz
sudo mv bearclaw /usr/local/bin/
```

### 通过 crates.io 安装

```bash
cargo install bearclaw
```

### 从源码编译

```bash
git clone https://github.com/xxieen/bearclaw.git
cd bearclaw
cargo install --path .
```

### 验证安装

```bash
bearclaw health
```

### 系统要求

- macOS（Bear 仅支持 macOS/iOS）
- 已安装 [Bear](https://bear.app)（写入操作需要 Bear 在运行中）

## 快速上手

```bash
# 检查工具是否正常工作
bearclaw health

# 搜索笔记
bearclaw search "rust programming"

# 读取指定笔记
bearclaw read <笔记ID>

# 创建笔记
bearclaw create --title "我的笔记" --body "你好世界" --tags "测试,示例"

# 查看统计信息
bearclaw stats
```

## 命令一览

### 笔记操作

| 命令 | 说明 |
|------|------|
| `bearclaw read <ID或标题>` | 读取笔记完整内容 |
| `bearclaw search <关键词>` | 搜索笔记（`--ocr`, `--tag`, `--since`, `--before`, `--limit`） |
| `bearclaw create --title "x" --body "y"` | 创建新笔记（`--tags`, `--body-file`） |
| `bearclaw edit <ID> --body "x"` | 替换笔记正文（`--body-file`） |
| `bearclaw append <ID> --text "x"` | 追加文本（`--header`, `--text-file`） |
| `bearclaw prepend <ID> --text "x"` | 在开头插入文本（`--text-file`） |
| `bearclaw section <ID> --header "x"` | 按标题提取某个章节内容 |
| `bearclaw trash <ID>` | 移到回收站 |
| `bearclaw archive <ID>` | 归档笔记 |

### 标签操作

| 命令 | 说明 |
|------|------|
| `bearclaw tag list` | 树形展示所有标签及笔记数 |
| `bearclaw tag add <ID> --tags "a,b"` | 给笔记添加标签 |
| `bearclaw tag rename <旧名> <新名>` | 重命名标签 |
| `bearclaw tag delete <标签名>` | 删除标签 |
| `bearclaw untagged` | 列出未打标签的笔记 |

### 分析与链接

| 命令 | 说明 |
|------|------|
| `bearclaw backlinks <ID或标题>` | 查找反向链接 |
| `bearclaw stats` | 显示统计信息 |

### 批量操作与导出

| 命令 | 说明 |
|------|------|
| `bearclaw batch tag --filter "关键词" --tags "a,b"` | 批量打标签 |
| `bearclaw batch archive --filter "关键词"` | 批量归档 |
| `bearclaw export --output ./目录/` | 导出为 Markdown（`--tag`, `--since`, `--before`） |

### 诊断

| 命令 | 说明 |
|------|------|
| `bearclaw health` | 检查 Bear 安装状态和数据库连接 |

### 全局选项

| 选项 | 说明 |
|------|------|
| `--pretty` | 美化 JSON 输出 |
| `--db-path <路径>` | 自定义数据库路径（或设置 `BEAR_DB_PATH` 环境变量） |

## JSON 输出格式

所有命令返回统一的 JSON 结构：

```json
// 成功
{"ok": true, "data": {...}, "count": 10}

// 失败
{"ok": false, "error": "Note not found", "code": "NOT_FOUND"}
```

## AI Agent 集成

Bearclaw 可与任何 AI 编程 Agent 配合使用。以下是各平台的配置方法。

> **前提条件**：请先安装 `bearclaw` CLI（参见上方[安装](#安装)部分）。

---

### Amp

从本仓库直接安装内置 Skill：

```bash
amp skill add xxieen/bearclaw/bear
```

当你向 Amp 询问 Bear 笔记相关问题时，Skill 会自动加载。首次使用会检查 `bearclaw` 是否已安装。

---

### Claude Code

在你项目的 `CLAUDE.md` 中添加以下内容：

```markdown
## Tools

You have access to `bearclaw`, a CLI tool for managing Bear notes on macOS.
All commands output JSON. Use `bearclaw <command> --help` for usage details.

Available commands:

- `bearclaw search <query>` — search notes (--ocr, --tag, --since, --before, --limit)
- `bearclaw read <id>` — read a note's full content
- `bearclaw create --title "x" --body "y" --tags "a,b"` — create a note
- `bearclaw edit <id> --body "x"` — replace note body
- `bearclaw append <id> --text "x"` — append text to a note
- `bearclaw tag list` — list all tags as tree
- `bearclaw tag add <id> --tags "a,b"` — add tags to a note
- `bearclaw untagged` — list untagged notes
- `bearclaw stats` — show statistics
- `bearclaw export --output ./dir/` — export notes as Markdown

Use note ID (not title) for write operations. Use `\n` for newlines in --body/--text.
```

---

## 架构

- **读取**：直接访问 SQLite 数据库（速度快，不需要 Bear 运行）
- **写入**：通过 Bear 官方 `x-callback-url` API（安全可靠，需要 Bear 运行中）
- **输出**：JSON 格式输出到 stdout，方便 Agent 解析

## 许可证

MIT
