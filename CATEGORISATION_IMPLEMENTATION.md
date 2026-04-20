# Implementation Specification: Topic Categorisation & Dashboard Redesign

Branch: `claude/improve-topic-dashboard-v1JXh`
Base: `main`

This document is a complete specification for reimplementing everything on this branch. It is ordered by layer (data → storage → MCP → dashboard → skills) so each section builds on the previous. All design decisions and interaction details are included.

---

## 1. Project Overview

**mentor-plugin** is a Socratic mentor system for coding agents. It uses SM-2 spaced repetition to track topic mastery over development sessions.

**Workspace:** `knowledge/` (Rust, 2024 edition)
- `learning/` — domain model and SQLite storage
- `mcp/` — MCP server exposing tools to Claude
- `dashboard/` — ratatui TUI dashboard
- `migrations/` — DB utilities

**Plugins:** `claude-plugin/` and `opencode-plugin/` — Markdown skill files consumed by Claude/OpenCode.

**Database:** SQLite at `~/.local/share/agent-mentor/knowledge.db` (configurable via `AGENT_MENTOR_STORAGE_FOLDER`).

---

## 2. Data Model Changes

### File: `knowledge/learning/src/topic.rs`

Add `categories: Vec<String>` field to `Topic`:

```rust
#[derive(Clone)]
pub struct Topic {
    pub name: String,
    pub repetitions: u32,
    pub interval: u32,
    pub ease_factor: f32,
    pub reviewed_at: u64,
    pub categories: Vec<String>,  // ← NEW
}
```

Update `Topic::new()` to initialise it:
```rust
pub fn new(name: &str, reviewed_at: u64) -> Topic {
    Topic {
        name: name.to_string(),
        repetitions: 0,
        interval: 1,
        ease_factor: 2.5,
        reviewed_at,
        categories: vec![],  // ← NEW
    }
}
```

### File: `knowledge/learning/src/sm2.rs`

The `sm2()` function constructs a new `Topic` — add the `categories` field so it is carried through:

```rust
Topic {
    name: topic.name.clone(),
    repetitions: updated_repetitions,
    interval: updated_interval,
    ease_factor: updated_ease_factor,
    reviewed_at: review_date,
    categories: topic.categories.clone(),  // ← NEW
}
```

Also update the `mocked_topic()` test helper to include `categories: vec![]`.

---

## 3. Storage Layer Changes

### File: `knowledge/learning/src/topic_storage.rs`

Add two new methods to the `TopicStorage` trait:

```rust
pub trait TopicStorage {
    fn get_overdue(&self, now: u64) -> Result<Vec<Topic>, StorageError>;
    fn get_all(&self) -> Result<Vec<Topic>, StorageError>;
    fn get(&self, topic: &str) -> Result<Option<Topic>, StorageError>;
    fn upsert(&self, topic: &Topic) -> Result<(), StorageError>;
    // NEW:
    fn set_topic_categories(&self, topic: &str, categories: &[String]) -> Result<(), StorageError>;
    fn get_categories(&self) -> Result<Vec<String>, StorageError>;
}
```

### File: `knowledge/learning/src/sqlite/sqlite_topic_storage.rs`

#### 3a. Schema — add two tables to `create_tables()`

```sql
PRAGMA foreign_keys = ON;
BEGIN;
CREATE TABLE IF NOT EXISTS topics ( ... existing ... );
CREATE TABLE IF NOT EXISTS categories (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  name TEXT NOT NULL UNIQUE COLLATE NOCASE
);
CREATE TABLE IF NOT EXISTS topic_categories (
  topic_id INTEGER NOT NULL REFERENCES topics(id) ON DELETE CASCADE,
  category_id INTEGER NOT NULL REFERENCES categories(id) ON DELETE CASCADE,
  PRIMARY KEY (topic_id, category_id)
);
COMMIT;
```

This is additive — existing databases gain the two new tables on first run.

#### 3b. Update `map_row_to_topic`

Column 5 (index 5) is now `GROUP_CONCAT(c.name, '|')` — parse it:

```rust
fn map_row_to_topic(&self, row: &Row) -> Result<Topic, rusqlite::Error> {
    let categories_str: Option<String> = row.get(5)?;
    let categories = categories_str
        .map(|s| s.split('|').map(String::from).collect())
        .unwrap_or_default();
    Ok(Topic {
        name: row.get(0)?,
        repetitions: row.get(1)?,
        interval: row.get(2)?,
        ease_factor: row.get(3)?,
        reviewed_at: row.get::<_, i64>(4)? as u64,
        categories,
    })
}
```

#### 3c. Update all three SELECT queries

Add a LEFT JOIN + GROUP_CONCAT to `get_all`, `get`, and `get_overdue`. Pattern:

```sql
SELECT t.name, t.repetitions, t.interval_days, t.ease_factor, t.reviewed_at,
       GROUP_CONCAT(c.name, '|') as categories
FROM topics t
LEFT JOIN topic_categories tc ON tc.topic_id = t.id
LEFT JOIN categories c ON c.id = tc.category_id
WHERE <existing_condition>
GROUP BY t.id
ORDER BY <existing_order>
```

`get_all` should order by `t.name COLLATE NOCASE`.

#### 3d. Implement `set_topic_categories`

```rust
fn set_topic_categories(&self, topic_name: &str, categories: &[String]) -> Result<(), StorageError> {
    let conn = self.0.lock().unwrap();
    conn.execute_batch("PRAGMA foreign_keys = ON;")?;

    let topic_id: i64 = conn.query_row(
        "SELECT id FROM topics WHERE name = ?1 COLLATE NOCASE",
        [topic_name], |row| row.get(0),
    )?;

    conn.execute("DELETE FROM topic_categories WHERE topic_id = ?1", [topic_id])?;

    for cat in categories {
        let cat = cat.trim();
        if cat.is_empty() { continue; }
        conn.execute("INSERT OR IGNORE INTO categories (name) VALUES (?1)", [cat])?;
        let cat_id: i64 = conn.query_row(
            "SELECT id FROM categories WHERE name = ?1 COLLATE NOCASE",
            [cat], |row| row.get(0),
        )?;
        conn.execute(
            "INSERT OR IGNORE INTO topic_categories (topic_id, category_id) VALUES (?1, ?2)",
            rusqlite::params![topic_id, cat_id],
        )?;
    }
    Ok(())
}
```

#### 3e. Implement `get_categories`

```rust
fn get_categories(&self) -> Result<Vec<String>, StorageError> {
    let conn = self.0.lock().unwrap();
    let mut stmt = conn.prepare("SELECT name FROM categories ORDER BY name COLLATE NOCASE")?;
    let cats = stmt.query_map([], |row| row.get(0))?
        .collect::<Result<Vec<String>, _>>()?;
    Ok(cats)
}
```

---

## 4. MCP Tool Changes

### New file: `knowledge/mcp/src/set_topic_categories.rs`

```rust
use rmcp::schemars;

#[derive(schemars::JsonSchema, serde::Deserialize)]
pub struct SetTopicCategoriesParams {
    pub topic: String,
    pub categories: Vec<String>,
}

#[derive(serde::Serialize)]
pub struct SetTopicCategoriesResult {
    pub topic: String,
    pub categories: Vec<String>,
}
```

### New file: `knowledge/mcp/src/list_all_topics.rs`

```rust
use rmcp::schemars;

#[derive(schemars::JsonSchema, serde::Deserialize)]
pub struct ListAllTopicsParams {}

#[derive(serde::Serialize)]
pub struct TopicEntry {
    pub name: String,
    pub categories: Vec<String>,
}
```

### File: `knowledge/mcp/src/main.rs`

Register both new modules:
```rust
mod list_all_topics;
mod set_topic_categories;
```

### File: `knowledge/mcp/src/tool_service.rs`

Import the new types and add two `#[tool]` methods to the `ToolService` impl. **Important:** `Parameters<T>` requires `T: JsonSchema` — always use `rmcp::schemars::JsonSchema` derive on params structs.

```rust
#[tool(description = "Assign one or more categories to a topic, replacing any previously \
    assigned categories. Creates categories that do not yet exist. Call this after \
    review_topic to keep topics organised.")]
async fn set_topic_categories(
    &self,
    params: Parameters<SetTopicCategoriesParams>,
) -> Result<String, String> {
    let topic_name = normalise_topic(&params.0.topic);
    self.topic_storage
        .set_topic_categories(&topic_name, &params.0.categories)
        .map_err(|e| e.to_string())?;
    let result = SetTopicCategoriesResult {
        topic: topic_name,
        categories: params.0.categories,
    };
    serde_json::to_string(&result).map_err(|e| e.to_string())
}

#[tool(description = "Return all topics with their assigned categories. Use this to get a \
    full picture of what has been studied, especially before running a categorisation pass.")]
async fn list_all_topics(
    &self,
    _params: Parameters<ListAllTopicsParams>,
) -> Result<String, String> {
    let topics = self.topic_storage.get_all().map_err(|e| e.to_string())?;
    let entries = topics.into_iter()
        .map(|t| TopicEntry { name: t.name, categories: t.categories })
        .collect::<Vec<_>>();
    serde_json::to_string(&entries).map_err(|e| e.to_string())
}
```

---

## 5. Dashboard Redesign

The dashboard has **two main views** — Topics (`t`) and Sessions (`s`). The Sessions view is unchanged. The Topics view is fully redesigned into a single unified interactive screen — no tabs.

### File layout after changes

```
knowledge/dashboard/src/
  main.rs         ← updated (routing, key handling)
  state.rs        ← fully rewritten
  topics_view.rs  ← new file (entire topics view)
  sessions.rs     ← unchanged
```

`topics.rs` and `categories.rs` are deleted.

---

### 5a. `state.rs` — full rewrite

#### Types

```rust
#[derive(Copy, Clone, PartialEq)]
pub enum View { Topics, Sessions }

#[derive(Copy, Clone, PartialEq)]
pub enum Zone { Stats, Categories, Topics }

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub enum StatFilter { Overdue, DueThisWeek, Mastered, Struggling }

#[derive(Copy, Clone)]
pub enum Pane { Sessions, SessionMd }  // sessions view panes, unchanged

pub const STAT_COUNT: usize = 4;
pub const STAT_FILTERS: [StatFilter; 4] = [
    StatFilter::Overdue, StatFilter::DueThisWeek,
    StatFilter::Mastered, StatFilter::Struggling,
];
pub const STAT_LABELS: [&str; 4] = ["Overdue", "Due this week", "Mastered", "Struggling"];
```

#### Model

```rust
pub struct Model {
    pub selected_view: View,
    pub topics: Vec<Topic>,
    pub sessions: Vec<Session>,

    // Topics view — zone focus
    pub focused_zone: Option<Zone>,

    // Stats zone cursor + active filters
    pub stats_cursor: usize,
    pub active_stat_filters: HashSet<StatFilter>,

    // Categories zone cursor + active filters
    pub categories_cursor: usize,
    pub active_category_filters: HashSet<String>,

    // Topics list scroll
    pub topics_state: TableState,

    // Sessions view (unchanged)
    pub session_state: TableState,
    pub focused_pane: Pane,
    pub session_md_scroll: u16,
}
```

#### Messages

```rust
pub enum Message {
    ShowTopicView,
    ShowSessionView,
    UpdateTopics(Vec<Topic>),
    UpdateSessions(Vec<Session>),
    FocusZone(Zone),   // pressing 1/2/3 — toggles if already focused
    ExitZone,          // Esc
    ToggleFilter,      // Space / Enter — toggles the highlighted item as a filter
    ResetFilters,      // r
    NextPane,          // l in sessions view
    PrevPane,          // h in sessions view
}
```

**`FocusZone` toggles:** if the pressed zone is already focused, calling `FocusZone(zone)` clears focus (`focused_zone = None`). This lets users press `1` again to unfocus.

#### Navigation functions (called directly from `main.rs`)

Navigation is context-sensitive so it is handled as functions rather than messages, since the Topics list navigation needs the current filtered count:

```rust
pub fn navigate_down(model: &mut Model, now: u64) { ... }
pub fn navigate_up(model: &mut Model) { ... }
pub fn navigate_left(model: &mut Model) { ... }   // Stats zone: move cursor left
pub fn navigate_right(model: &mut Model) { ... }  // Stats zone: move cursor right
```

Behaviour table:

| View | Zone | j/k | h/l |
|---|---|---|---|
| Topics | Stats | moves stats cursor down/up | moves stats cursor left/right |
| Topics | Categories | scrolls category list | — |
| Topics | Topics or none | scrolls filtered topic list | — |
| Sessions | any | scrolls session list / md scroll | switches pane |

#### Helper functions (public, used by `topics_view.rs`)

```rust
// Returns all category names sorted alphabetically, "Uncategorized" last.
pub fn sorted_categories(topics: &[Topic]) -> Vec<String>

// Returns references to topics matching ALL active filter groups.
// Within each group (stat filters, category filters), matching is OR.
// Between groups it is AND.
// Empty group = match all.
pub fn filtered_topics<'a>(
    topics: &'a [Topic],
    stat_filters: &HashSet<StatFilter>,
    cat_filters: &HashSet<String>,
    now: u64,
) -> Vec<&'a Topic>
```

**Filter semantics:**
- `active_stat_filters` = OR within: topic matches if it satisfies *any* selected stat
- `active_category_filters` = OR within: topic matches if it belongs to *any* selected category
- Between the two groups: AND — topic must satisfy both groups

---

### 5b. `topics_view.rs` — new file

This file owns the entire Topics view. It also exports navigation helpers used by `state.rs`:

```rust
pub fn update_selected_table_up(state: &mut TableState)
pub fn update_selected_table_down(state: &mut TableState, len: usize)
pub fn epoch_now() -> u64
pub fn render_topics_view(frame: &mut Frame, area: Rect, model: &mut Model)
```

#### Layout

```
area
├── stats_area     Length(3)           — stats bar
├── chart_area     Length(clamp(cats+2, 4, 12))  — categories chart
└── list_area      Fill(1)             — filtered topics list
```

`cats` = number of unique categories. The chart height is clamped so the topics list always has space.

#### Stats bar

5 equal horizontal columns: **Total** (non-interactive) + **4 filterable stats**.

Each stat card is a `Paragraph` inside a `Block::bordered()`. Visual states:

| State | Border | Background | Text |
|---|---|---|---|
| Normal | Plain, default | Reset | default |
| Zone focused (thick borders on all cards) | Thick | — | — |
| Cursor on card | Thick + Cyan | DarkGray | default |
| Active filter | default color | Reset | colored + bold |
| Cursor + active | Thick + Cyan | DarkGray | colored + bold |

When the Stats zone is focused (`focused_zone == Some(Zone::Stats)`), all 5 card borders switch to `BorderType::Thick` and the cursor card gets `Color::Cyan` border + `DarkGray` background.

Stat value colors: Overdue=Red, DueThisWeek=Yellow, Mastered=Green, Struggling=Red.

#### Categories chart

A `Block::bordered()` containing manually rendered lines (one per category). The block title shows active category filters when any are selected.

When focused, border is `BorderType::Thick` with `Color::Cyan`.

**Bar calculation:**
```
max_count  = max topics in any single category
bar_max    = inner.width - name_col(20) - count_col(5) - 1
bar_total  = (count * bar_max) / max_count
bar_master = (mastered * bar_total) / count
bar_learn  = (learning * bar_total) / count
bar_strug  = bar_total - bar_master - bar_learn
```

**Characters used:**
- `█` (U+2588) — mastered → `Color::Green`
- `▓` (U+2593) — learning → `Color::Yellow`
- `░` (U+2591) — struggling → `Color::Red`

**Scrolling:** when `categories_cursor >= visible_rows`, the scroll offset is `cursor - visible_rows + 1`. Categories render from `scroll_offset` for `visible_rows` lines.

**Row states:**

| State | Background | Name style |
|---|---|---|
| Normal | Reset | default |
| Cursor (zone focused) | DarkGray | default |
| Active filter | Reset | Cyan + Bold |
| Cursor + active | DarkGray | Cyan + Bold |

#### Topics list

A `Block::bordered()` containing a stateful `Table`. The block title shows the filter summary:
- No filters: ` Topics (25) `
- With filters: ` Topics (4/25) · Mastered, Rust `

When focused or no zone is focused, border is `BorderType::Thick` + `Color::Cyan`.

**Table columns:**
| Column | Constraint |
|---|---|
| Topic name | Percentage(27) |
| Categories (comma-separated, `-` if none) | Percentage(22) |
| Ease factor (1 decimal) | Percentage(6) |
| Repetitions | Percentage(5) |
| Next review date | Percentage(40) |

**Row color coding:**

| Condition | Color |
|---|---|
| `next_review < now` (overdue) | Red |
| `question_depth() == Skip` (mastered) | Green |
| `question_depth() == Light` (learning) | Yellow |
| otherwise | default |

Overdue takes priority. Next review date shows `(overdue)` suffix if past due.

---

### 5c. `main.rs` — updated

#### Two views, no tabs

```
header: Length(1)   — title + hint line
main:   Fill(1)     — bordered content block
```

No tab bar. The header line shows view-specific hints:
- Topics: `Mentor [Topics] (q) quit · (s) sessions · (1) stats · (2) categories · (3) topics · (Space) filter · (r) reset · (Esc) unfocus`
- Sessions: `Mentor [Sessions] (q) quit · (t) topics · (j/k) navigate · (h/l) pane`

#### Key bindings — Topics view

| Key | Action |
|---|---|
| `q` | quit |
| `s` | switch to Sessions view |
| `1` | `FocusZone(Stats)` (toggle) |
| `2` | `FocusZone(Categories)` (toggle) |
| `3` | `FocusZone(Topics)` (toggle) |
| `Esc` | `ExitZone` |
| `r` | `ResetFilters` |
| `Space` or `Enter` | `ToggleFilter` |
| `j` or `↓` | `navigate_down(model, now)` |
| `k` or `↑` | `navigate_up(model)` |
| `h` or `←` | `navigate_left(model)` |
| `l` or `→` | `navigate_right(model)` |

#### Key bindings — Sessions view

| Key | Action |
|---|---|
| `q` | quit |
| `t` | switch to Topics view |
| `j` or `↓` | navigate down |
| `k` or `↑` | navigate up |
| `h` or `←` | `PrevPane` |
| `l` or `→` | `NextPane` |

---

## 6. Skill Changes

### New file: `claude-plugin/skills/mentor+-categorize/SKILL.md`
### New file: `opencode-plugin/skills/mentor+-categorize/SKILL.md`

Both files are identical. This is a new user-invocable skill (`/mentor+categorize`).

**Workflow:**
1. Call `list_all_topics` → show summary (total, categorised, uncategorised count)
2. Propose categories in a table: `Topic | Proposed categories`. Show `(unchanged)` for topics that already have correct categories
3. Ask user to confirm or request corrections
4. Apply any corrections, show updated table if changed
5. Call `set_topic_categories` for every topic that needs updating (skip unchanged)
6. Report: how many updated + final category breakdown with counts

**Category naming rules:**
- Broad, stable domain names: `Rust`, `Frontend`, `Algorithms`, `Databases`, `Architecture`, `Testing`, `DevOps`, `Python`, `TypeScript`, `Concurrency`, `Security`, `Design Patterns`
- Title case
- 1–3 categories per topic maximum
- Prefer reusing existing category names over creating new ones

---

### Updated file: `claude-plugin/skills/mentor+/SKILL.md`
### Updated file: `opencode-plugin/skills/mentor+/SKILL.md`

Two changes:

1. Update tool count from "five" to "seven" in the Knowledge Tracking header.

2. Add an **Auto-categorisation** section after the `review_topic` description:

> **Auto-categorisation**
>
> Immediately after calling `review_topic` for a topic that has no categories (newly created this session, or confirmed empty via `list_all_topics`), call `set_topic_categories` with 1–3 inferred categories.
>
> Rules:
> - Use broad title-case domain names: `Rust`, `Frontend`, `Algorithms`, `Databases`, `Architecture`, `Testing`, `DevOps`, `Python`, `TypeScript`, `Concurrency`, `Security`, `Design Patterns`
> - 1 category is fine, 3 is the maximum
> - Do not create narrow categories (avoid `Rust Closures` — use `Rust`)
> - Do not re-categorise topics that already have categories

---

## 7. Summary of All Changed Files

| File | Change |
|---|---|
| `knowledge/learning/src/topic.rs` | Add `categories: Vec<String>` to `Topic`, init in `new()` |
| `knowledge/learning/src/sm2.rs` | Carry `categories` through in `sm2()`, fix test helper |
| `knowledge/learning/src/topic_storage.rs` | Add `set_topic_categories` and `get_categories` to trait |
| `knowledge/learning/src/sqlite/sqlite_topic_storage.rs` | Add 2 tables, update all SELECT queries, implement new methods |
| `knowledge/mcp/src/set_topic_categories.rs` | **New** — params/result structs |
| `knowledge/mcp/src/list_all_topics.rs` | **New** — params/result structs |
| `knowledge/mcp/src/main.rs` | Register 2 new modules |
| `knowledge/mcp/src/tool_service.rs` | Add 2 `#[tool]` methods |
| `knowledge/dashboard/src/state.rs` | **Fully rewritten** — zones, filters, helpers |
| `knowledge/dashboard/src/topics_view.rs` | **New** — entire Topics view |
| `knowledge/dashboard/src/main.rs` | **Rewritten** — two views, zone key handling |
| `knowledge/dashboard/src/topics.rs` | **Deleted** |
| `knowledge/dashboard/src/categories.rs` | **Deleted** |
| `claude-plugin/skills/mentor+/SKILL.md` | Add auto-categorisation section, update tool count |
| `claude-plugin/skills/mentor+-categorize/SKILL.md` | **New** skill |
| `opencode-plugin/skills/mentor+/SKILL.md` | Same as claude-plugin version |
| `opencode-plugin/skills/mentor+-categorize/SKILL.md` | **New** skill |

---

## 8. Key Design Decisions

**Many-to-many categories** — A topic can belong to multiple categories (e.g. "Rust lifetimes" → `Rust` + `Memory Management`). Stored in a proper junction table, not a delimited string column.

**`GROUP_CONCAT` for reads** — Rather than N+1 queries, categories are aggregated with `GROUP_CONCAT(c.name, '|')` in a single JOIN query. The `|` delimiter is chosen because it cannot appear in a category name.

**Filter semantics: OR within group, AND between groups** — Selecting `Overdue + Rust` means "overdue topics that are also in Rust". Selecting `Overdue + Mastered` means "topics that are either overdue or mastered". This matches the mental model of: stat filters broaden within a dimension, category filters narrow across dimensions.

**Zone focus toggles** — Pressing `1` while Stats is already focused unfocuses it. This allows keyboard-only flow without requiring `Esc`.

**Chart height clamped 4–12** — The categories chart is `clamp(num_categories + 2, 4, 12)` rows. This ensures the topics list is always visible even with many categories. Categories scroll within the chart when there are more than fit.

**Auto-migration** — The `CREATE TABLE IF NOT EXISTS` pattern means existing databases gain the new tables silently on first launch. No manual migration step needed.

**`set_topic_categories` is a full replace** — Calling it deletes all existing links for the topic before inserting new ones. This makes it idempotent and avoids duplicate handling.
