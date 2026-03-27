# Release Strategy

This project provides a Socratic mentor plugin for agentic coding tools. It ships:

- **Skills** — Markdown instruction files loaded by the coding agent
- **MCP server** — A Rust binary exposing knowledge-tracking tools via the Model Context Protocol
- **Hooks** — Session lifecycle scripts (Claude Code only)

## Branch-based distribution

Each supported coding agent gets its own release branch containing only the files it needs. This avoids users having to clone the full repository.

| Agent | Release branch |
|---|---|
| Claude Code | `claude-plugin-release` |
| OpenCode | `opencode-plugin-release` |

The CI pipeline pushes to these branches automatically on every version tag.

## What goes where

### Skills
Copied from `claude-plugin/skills/` or `opencode-plugin/skills/` into the corresponding release branch as-is.

### Agent configuration
Agent-specific config files are copied into the release branch. Examples:
- `.mcp.json` — Claude Code MCP server registration
- `opencode.json` — OpenCode MCP server registration

### MCP server script
`scripts/mcp-server.sh` is copied into the release branch. It handles downloading the correct binary for the user's OS/arch on first run.

### MCP binary
The Rust binary is compiled for each supported platform and published as GitHub release assets. The `mcp-server.sh` script downloads the right binary automatically and caches it at:

```
~/.local/bin/mentor-mcp-<version>
```

Supported platforms: `linux-x86_64`, `darwin-arm64`.
