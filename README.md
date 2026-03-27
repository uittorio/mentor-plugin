# mentor-plugin

Turns your coding agent into a Socratic mentor for learning projects.

Instead of handing you answers, the agent guides you with questions, hints, and explanations — building real understanding rather than dependency.

Once installed, mentor mode is **always on** for that project. The agent will never hand you solutions — it will guide you to find them yourself.

---

## Skills

The plugin ships two skills. Install the one that fits your setup.

### `mentor`

Socratic mentoring with persistent session recaps written to your [Obsidian](https://obsidian.md) vault.

- Guides with questions instead of answers
- Tracks errors, wins, and hard points throughout the session
- Silently maintains a session draft — no data lost if the session ends abruptly
- On session end, writes a structured recap to your vault (what was worked on, misconceptions, wins, suggested reading, next steps)
- References past session notes via Obsidian wikilinks when a pattern repeats

### `mentor+`

Socratic mentoring with spaced repetition knowledge tracking via MCP.

- Same Socratic approach as `mentor`
- At session start, checks your knowledge level per topic and adjusts questioning depth accordingly
- Records learning outcomes after each meaningful exchange
- Surfaces topics due for review at the start of each session

> Obsidian vault integration is not yet supported in `mentor+`.

---

## Supported agents

| Agent | `mentor` | `mentor+` |
|---|---|---|
| [Claude Code](https://claude.ai/code) | ✓ | ✓ |
| [OpenCode](https://opencode.ai) | ✓ | ✓ |

Both macOS and Linux are supported.

→ **[Installation instructions](./INSTALLATION.md)**

---

## Usage

No invocation needed — mentor mode activates automatically when the agent detects you are learning something. Just start working.

Tip: mention your background at the start of a session so the agent can bridge concepts more effectively:

```
I'm learning Go, coming from TypeScript.
```

### First session setup (`mentor`)

On your first session, the agent will ask two things in a single message:

1. **Vault path** — paste the absolute path to your Obsidian vault (e.g. `/home/you/MyVault`) or say `skip`
2. **Your background** — what you're learning, your programming experience, and your goals

These are saved to `.claude/mentor-config.json` in your project and reused in every future session.

---

## License

MIT
