# mentor-plugin

Turns your coding agent into a Socratic mentor for learning projects — with spaced repetition knowledge tracking built in.

Instead of handing you answers, the agent guides you with questions, hints, and explanations — building real understanding rather than dependency. It remembers what you know and what you struggle with across sessions, adjusting how hard it pushes you on each topic.

Once enabled, mentor mode is **always on** for that session. The agent will never hand you solutions — it will guide you to find them yourself.

---

## What it does

- Guides with questions instead of answers
- At session start, checks your knowledge level per topic and adjusts questioning depth accordingly
- Records learning outcomes after each meaningful exchange
- Surfaces topics due for review at the start of each session
- Writes a running draft file during the session capturing every learning exchange and notable moment
- On demand, generates a structured end-of-session summary prepended to the draft

---

## How it works

A few components wire together when installed: 
- a **skill** (mentor+) to shapes how the agent teaches
- an **MCP server** that tracks what you know.
- a **skill** (mentor+summarise) to summarise your session

### mentor+

```mermaid
graph LR
    classDef agent fill:#dbeafe,stroke:#3b82f6,color:#1e40af
    classDef skill fill:#fef3c7,stroke:#f59e0b,color:#92400e
    classDef mcp fill:#d1fae5,stroke:#10b981,color:#065f46
    classDef db fill:#f3f4f6,stroke:#6b7280,color:#374151
    classDef tool fill:#fce7f3,stroke:#ec4899,color:#831843
    classDef file fill:#ede9fe,stroke:#7c3aed,color:#4c1d95

    A["🤖 Agent\n(Claude Code / OpenCode)"]:::agent -->|activates| SK(["📖 mentor+ Skill\nSocratic method"]):::skill
    SK -->|MCP calls| MCP["⚙️ MCP Server\nRust · SM-2"]:::mcp
    MCP --- DB[("🗄️ knowledge.db")]:::db
    SK -->|appends after each exchange| DRAFT[("📝 session draft\n.md file")]:::file

    MCP --> T1["get_topic_candidates\ndue topics at session start"]:::tool
    MCP --> T2["get_topics\nfuzzy-match existing topics"]:::tool
    MCP --> T3["topic_depth\nfull · light · skip"]:::tool
    MCP --> T4["review_topic\nrecord outcome · update interval"]:::tool
    MCP --> T5["create_session\ncreate session · return draft path"]:::tool
```

### mentor+summarise

```mermaid
graph LR
    classDef agent fill:#dbeafe,stroke:#3b82f6,color:#1e40af
    classDef skill fill:#fef3c7,stroke:#f59e0b,color:#92400e
    classDef file fill:#ede9fe,stroke:#7c3aed,color:#4c1d95
    classDef source fill:#f3f4f6,stroke:#6b7280,color:#374151

    A["🤖 Agent\n(Claude Code / OpenCode)"]:::agent -->|activates| SK(["📋 mentor+-summarise Skill"]):::skill
    SK -->|reads| DRAFT[("📝 session draft\n.md file")]:::file
    SK -->|enriches from| CH["conversation history"]:::source
    SK -->|enriches from| GIT["git log"]:::source
    SK -->|enriches from| CODE["referenced files"]:::source
    SK -->|prepends summary to| DRAFT
```

---

## Supported coding agents

| Agent | Support |
|---|---|
| [Claude Code](https://claude.ai/code) | ✓ |
| [OpenCode](https://opencode.ai) | ✓ |

Both macOS and Linux are supported.

→ **[Installation instructions](./INSTALLATION.md)**

---

## Usage

### Starting a session

#### Claude Code
```
/mentor+
```

#### OpenCode
Run `/skills` and select `mentor+`.

---

### Ending a session

When you're done, generate a structured summary of what was learned:

#### Claude Code
```
/mentor+summarise
```

#### OpenCode
Run `/skills` and select `mentor+summarise`.

The summariser reads the session draft file and prepends a structured summary covering what was learned, what was struggled with, and what was built.


## Dashboard 
You can also visualise your historical topics and sessions to see your progress.

See [Installation](./INSTALLATION.md#dashboard) for more details

![dashboard-topics](./images/dashboard-topics.png)

![dashboard-sessions](./images/dashboard-sessions.png)
---


## Developer
Documentation about how this project works internally
[Developer](./DEVELOPER.md)

## License

MIT
