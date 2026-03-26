# mentor-plugin

A plugin that turns the code agent into a Socratic mentor for learning projects. Available for Claude and Opencode

Instead of handing you answers, the code agent will guide you with questions, hints, and explanations — building understanding rather than dependency.

# Claude installation

At the end of every session it writes a structured learning recap to your Obsidian vault automatically, even if the session ends unexpectedly.

> **Always on.** Once installed in a project, mentor mode is active for every message in that project. Claude will never hand you solutions — it will guide you to find them yourself.

## Install

Install into the project you are using for learning. Mentor mode is scoped to that project — Claude behaves normally everywhere else.

**Step 1 — Add the marketplace:**
```
/plugin marketplace add uittorio/mentor-plugin
```

**Step 2 — Install the plugin:**
```
/plugin install mentor-plugin@mentor-plugins
```

**Requirement:** `python3` must be available on your `$PATH` (used by the session-end hook to parse JSON).

## Usage

No invocation needed — mentor mode activates automatically when Claude detects you are learning something. Just start working.

Tip: mention your background at the start of a session so Claude can bridge concepts more effectively:

```
I'm learning Go, coming from TypeScript.
```

### First session setup

On your first session, Claude will ask two things in a single message:

1. **Vault path** — paste the absolute path to your Obsidian vault (e.g. `/home/you/MyVault`) or say `skip`
2. **Your background** — what you're learning, your programming experience, and your goals

These are saved to `~/.claude/mentor-config.json` and reused in every future session — you won't be asked again.

### Session recaps (Obsidian)

Once configured, Claude silently updates a draft recap at `~/.claude/mentor-session-draft.md` after every significant moment — error caught, hard point flagged, win noted, reading recommendation made. The draft is always current, so no data is lost if the session ends abruptly.

When the session ends — for **any** reason (graceful close, crash, `Ctrl+C`, tab close) — a `SessionEnd` hook automatically saves the final recap to:

```
<your-vault>/<topic-folder>/recaps/YYYY-MM-DD <topic>.md
```

If two sessions happen on the same day, the new recap is appended with a separator. If the session ended unexpectedly, the file is stamped with a note so you know to review it.

If you skip the vault, Claude prints the recap as plain text at the end of each session.

## What it does

**Mentoring**
- Never gives direct solutions — guides with questions and hints
- Asks what you've already tried before correcting
- Explains the *why* behind corrections, not just the *what*
- Bridges new concepts from your existing language knowledge
- Flags repeated mistakes explicitly after the second occurrence
- References past session recaps via Obsidian wikilinks when a pattern repeats

**Learning material**
- Surfaces canonical reading inline when a concept trips you up or your mental model is off
- Recommends official docs first, then well-regarded books or interactive resources
- Points to the exact section or chapter — not just a homepage

**Session recap (written to Obsidian)**
- What was worked on
- Errors & misconceptions — root cause and how each was resolved
- Wins
- Concepts to revisit (with reason why)
- Suggested reading — mapped to hard points from the session
- Next steps
- Mentor notes — a short observation on your learning pattern for the session

## Requirements

- Claude Code with plugin support (v2.1.x or later)
- `python3` on `$PATH`
- An Obsidian vault (optional, but recommended)

## License

MIT
