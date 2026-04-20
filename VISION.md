# Mentor Plugin — Vision

## Goal

Make this tool free and useful to the community — a different way to learn, grounded in real-world context and shared human experience.

---

## The Core Idea

Learning is most valuable when it's grounded in real problems. Someone debugging a production issue brings rich context: a real codebase, a real design pattern, a real failure mode. That context is currently lost. This platform captures it, abstracts it, and makes it reusable.

**The session is the unit of value.** Not just the topic, not just the answer — the *journey to understanding*.

---

## Current Data Model (SM-2 Topics)

The MCP server currently stores per-topic SM-2 scheduling state:

| Field | Description |
|---|---|
| `name` | Topic identifier |
| `repetitions` | Consecutive high-quality reviews |
| `interval_days` | Days until next review |
| `ease_factor` | Mastery level (increases with quality) |
| `reviewed_at` | Unix timestamp of last review |
| `categories` | Comma-separated knowledge domain labels |

This tells you *when* to review and *how well* you know something. It does not capture *what happened* during the learning exchange.

---

## What Needs to Be Added

### Sessions

A session groups one or more topics reviewed together (topics naturally touch each other). One session → many topics.

**Session record:**
- `id`
- `started_at`, `ended_at`
- `summary` — AI-generated: what was learned, what was struggled with, what was immediate
- `published` — whether the session is publicly discoverable
- `topics[]` — list of topic nodes covered

### Session Topic Nodes (the Learning Tree)

Each topic within a session is a tree, not a flat record. Nodes represent Socratic exchanges:

```
[ROOT NODE] — initial question on the topic
  ├── [quality HIGH 4-5] → move to next topic
  └── [quality LOW 0-2] → drill deeper
         ├── remediation question 1
         └── remediation question 2
               └── deeper remediation...
```

**Node record:**
- `topic_name`
- `question_type`: one of `clarifying | probing_assumptions | probing_reasoning | implications | alternatives`
- `quality_score` (0–5, SM-2 scale)
- `emergent`: boolean — was this node triggered organically by the learner (not planned by the tree)?
- `parent_node_id` — null for root nodes

### Session Summary (AI-Generated)

At the end of each session, an LLM generates:
- **Learned** — concepts confidently demonstrated
- **Struggled** — concepts that required deep remediation
- **Learned immediately** — high quality on first question (no drilling needed)

---

## Replay System

### The Goal

Allow someone else to go through a *similar* learning journey — not an identical one, but one that covers the same concepts, surfaces the same types of struggles, and adapts to their own level.

### What Gets Replayed

- The **outcome** (set of topics and concepts covered)
- The **how** (the type of Socratic path taken)
- Emergent topics are included in the replay curriculum

### What Does NOT Get Replayed

- Specific code examples (privacy)
- Exact conversation text
- The original learner's answers

### Replay Engine Design

The MCP server acts as a **traversal state machine** — the LLM drives the conversation, the MCP drives the tree:

1. LLM calls `start_replay(session_id)` → MCP loads the session summary (for context) and returns the first node `{ topic, question_type }`
2. LLM generates a Socratic question based on `question_type` and `topic`, presents it to the learner
3. LLM assesses the learner's response, calls `next_node(session_id, node_id, quality_score)` → MCP uses quality to decide traversal (high → skip children, low → drill deeper) and returns the next node
4. Repeat until tree is exhausted

**MCP state between calls:** `session_id` + `current_node_id` — enough to reconstruct position in the tree.

Emergent topics are presented as additional curriculum items after the planned tree is exhausted.

### Privacy & Abstraction

- Real code examples are **AI-abstracted** into generic equivalents before publishing
- Users **manually review** what goes public before a session is published
- Users control which sessions are discoverable

---

## Dashboard

An interactive TUI dashboard for visualising learning progress across topics and sessions.

### Topics view
- Stats bar showing overdue, due this week, mastered, and struggling counts — filterable
- Categories chart with per-category bar graphs showing mastered/learning/struggling breakdown
- Filtered topics list — filters combine across stat and category dimensions
- Colour coding: red for overdue, green for mastered, yellow for learning

### Sessions view
- List of past sessions with a markdown preview pane

---

## Future: Community Features

- Public session search — find sessions by topic, difficulty, date
- Replay a published session — go through someone else's learning journey
- Sessions as "production war stories" — real-world context abstracted and shared

---

## Session Schema (Designed, Not Yet Built)

### `sessions` table
| Field | Description |
|---|---|
| `id` | Primary key |
| `started_at` | Unix timestamp |
| `ended_at` | Unix timestamp (null until finalised) |
| `title` | Short description for display |
| `private_summary` | Full narrative with real context (md file path or text) |
| `public_summary` | AI-abstracted version safe to publish |
| `published` | Whether discoverable by others |

### `session_nodes` table
| Field | Description |
|---|---|
| `id` | Primary key |
| `session_id` | FK to sessions |
| `topic_name` | Topic covered |
| `question_type` | clarifying / probing_assumptions / probing_reasoning / implications / alternatives |
| `quality_score` | 0–5 SM-2 quality |
| `emergent` | true if learner introduced this topic organically |
| `parent_node_id` | null for root nodes |

Topics per session are derived via `SELECT DISTINCT topic_name FROM session_nodes WHERE session_id = ?` — no separate join table needed.

### Session lifecycle
- **Start:** `mentor+` skill creates a session row and draft file on activation
- **During:** each `review_topic` call appends a node; draft file updated with running summary
- **End:** user explicitly invokes an end-session command; AI generates both summaries; user reviews public version before publishing

---

## Architecture Notes

- DB: SQLite at `~/.local/share/agent-mentor/knowledge.db`
- MCP server: Rust binary at `knowledge/mcp/`
- Dashboard: Ratatui TUI binary `mentor-dashboard` at `knowledge/dashboard/`
- Session drafts: MD files alongside the database
