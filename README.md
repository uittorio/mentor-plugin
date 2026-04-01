# mentor-plugin

Turns your coding agent into a Socratic mentor for learning projects — with spaced repetition knowledge tracking built in.

Instead of handing you answers, the agent guides you with questions, hints, and explanations — building real understanding rather than dependency. It remembers what you know and what you struggle with across sessions, adjusting how hard it pushes you on each topic.

Once installed, mentor mode is **always on** for that project. The agent will never hand you solutions — it will guide you to find them yourself.

---

## What it does

- Guides with questions instead of answers
- At session start, checks your knowledge level per topic and adjusts questioning depth accordingly
- Records learning outcomes after each meaningful exchange
- Surfaces topics due for review at the start of each session

---

## How it works

Two components wire together when installed: a **skill** that shapes how the agent teaches, and an **MCP server** that tracks what you know.

```mermaid
graph LR
    classDef agent fill:#dbeafe,stroke:#3b82f6,color:#1e40af
    classDef skill fill:#fef3c7,stroke:#f59e0b,color:#92400e
    classDef mcp fill:#d1fae5,stroke:#10b981,color:#065f46
    classDef db fill:#f3f4f6,stroke:#6b7280,color:#374151
    classDef tool fill:#fce7f3,stroke:#ec4899,color:#831843

    A["🤖 Agent\n(Claude Code / OpenCode)"]:::agent -->|activates| SK(["📖 mentor+ Skill\nSocratic method"]):::skill
    SK -->|MCP calls| MCP["⚙️ MCP Server\nRust · SM-2"]:::mcp
    MCP --- DB[("🗄️ knowledge.db")]:::db

    MCP --> T1["get_topic_candidates\ndue topics at session start"]:::tool
    MCP --> T2["get_topics\nfuzzy-match existing topics"]:::tool
    MCP --> T3["topic_depth\nfull · light · skip"]:::tool
    MCP --> T4["review_topic\nrecord outcome · update interval"]:::tool
```

---

## Supported agents

| Agent | Support |
|---|---|
| [Claude Code](https://claude.ai/code) | ✓ |
| [OpenCode](https://opencode.ai) | ✓ |

Both macOS and Linux are supported.

→ **[Installation instructions](./INSTALLATION.md)**

---

## Usage

### Claude
Just use this command to start the session
```/mentor+```

### Opencode
Search for skills
```/skills```

Select mentor+

---

## License

MIT
