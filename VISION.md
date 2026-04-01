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

The session tree acts as a **curriculum template**:
1. For each root topic node, generate a new Socratic question of the same `question_type`
2. Use quality score to decide traversal: high quality → skip children; low quality → drill into children
3. Emergent topics are presented as additional curriculum items, not triggered organically (trade-off: some naturalness is lost)

### Privacy & Abstraction

- Real code examples are **AI-abstracted** into generic equivalents before publishing
- Users **manually review** what goes public before a session is published
- Users control which sessions are discoverable

---

## Dashboard (Immediate Goal)

A simple read-only view of the existing topic database:

- Table with all topics
- Columns: name, ease factor, interval (next review in N days), repetitions, last reviewed
- Search by topic name
- Sortable by any column
- Highlight overdue topics (interval elapsed)

No sessions data yet — that requires schema extension.

---

## Future: Community Features

- Public session search — find sessions by topic, difficulty, date
- Replay a published session — go through someone else's learning journey
- Sessions as "production war stories" — real-world context abstracted and shared
- Skill categorisation — group topics under existing plugin skills

---

## Architecture Notes

- DB: SQLite at `~/.local/share/agent-mentor/knowledge.db`
- MCP server: Rust binary (`mcp/sm2/`)
- Schema extension needed: `sessions` table, `session_nodes` table
- Dashboard: TBD (could be a standalone web app, a CLI command, or a plugin feature)
