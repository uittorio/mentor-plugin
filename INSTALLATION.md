# Installation

Releases are distributed via dedicated branches — see [Release Strategy](./RELEASE_STRATEGY.md) for details.

---

# Plugin

## OpenCode
The install plugin will copy the nececessary files from the release branch in your opencode configuration folder

### Global config (`~/.config/opencode/`)
```bash
curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/opencode-plugin-release/install.sh | bash
```

## Claude Code

**Step 1 — Add the marketplace:**
```
/plugin marketplace add uittorio/mentor-plugin@claude-plugin-release
```

**Step 2 — Install the plugin:**
```
/plugin install mentor-plugin@mentor-plugins
```

---

# Updates

## OpenCode
Re-run the install script — it always pulls the latest version from the release branch.

## Claude Code
Claude Code handles updates automatically. Run `/marketplace`, find the plugin, and update.

---

# Uninstall

## Claude Code

Remove the plugin:
```
/plugin uninstall mentor-plugin
```

Remove the binary and knowledge database:
```bash
rm -f ~/.local/bin/mentor-mcp
rm -rf ~/.local/share/agent-mentor
```

## OpenCode

Remove the skills:
```bash
rm -rf ~/.config/opencode/skills/mentor+
rm -rf ~/.config/opencode/skills/mentor+flow
rm -rf ~/.config/opencode/skills/mentor+summarise
rm -rf ~/.config/opencode/skills/mentor+categorize
```

Remove the MCP server script:
```bash
rm -rf ~/.config/opencode/agent-mentor
```

Remove the `agent-mentor` entry from your `opencode.json`

Remove the binary and knowledge database:
```bash
rm -f ~/.local/bin/mentor-mcp
rm -rf ~/.local/share/agent-mentor
```

## What gets installed and where

### Claude Code

Managed by the Claude Code plugin system. Files land wherever Claude Code stores the plugin, plus two shared locations written on first use:

| File | Destination |
|------|-------------|
| `SKILL.md` | inside the Claude Code plugin directory |
| `.mcp.json` | inside the Claude Code plugin directory |
| `mcp-server.sh` | inside the Claude Code plugin directory |
| MCP binary (downloaded on first run) | `~/.local/bin/mentor-mcp` |
| Knowledge database (created on first run) | `~/.local/share/agent-mentor/knowledge.db` |

### OpenCode

| File | Destination |
|------|-------------|
| `SKILL.md` | `~/.config/opencode/skills/mentor+/SKILL.md` |
| `SKILL.md` | `~/.config/opencode/skills/mentor+flow/SKILL.md` |
| `SKILL.md` | `~/.config/opencode/skills/mentor+summarise/SKILL.md` |
| `SKILL.md` | `~/.config/opencode/skills/mentor+categorize/SKILL.md` |
| `mcp-server.sh` | `~/.config/opencode/agent-mentor/mcp-server.sh` |
| MCP config entry | added to `~/.config/opencode/opencode.json` |
| MCP binary (downloaded on first run) | `~/.local/bin/mentor-mcp` |
| Knowledge database (created on first run) | `~/.local/share/agent-mentor/knowledge.db` |


# Dashboard

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/main/scripts/install-dashboard.sh | bash
```

Add ~/.local/bin to your PATH

## What gets installed and where

| File | Destination |
|------|-------------|
| Dashboard binary (downloaded on first run) | `~/.local/bin/mentor-dashboard` |

## Configuration (optional)

You can configure the dashboard by creating a `dashboard.toml` file in the knowledge data folder:

```
~/.local/share/agent-mentor/dashboard.toml
```

### Review topic commands

When you press `Enter` on a topic in the dashboard, it launches a review session. You can configure one or more commands to open that session — and rotate between them with `a`.

Each command has a `name` (shown in the dashboard) and `args` (the binary and its arguments). Use `$MentorPrompt` as a placeholder — the dashboard replaces it with the prompt for the selected topic.

Example for Mac using iTerm2 and Claude
```toml
[[review_topic_commands]]
name = "claude"
args = [
  "/usr/bin/osascript",
  "-e",
'''
  tell application "iTerm2"
      activate
      set newWindow to (create window with default profile)
      tell current session of newWindow
          write text "cd your/preferred/dir && /opt/homebrew/bin/claude \"$MentorPrompt\""
      end tell
  end tell
'''
]
```.

# Uninstall

Remove the mentor dashboard binary:
```bash
rm -f ~/.local/bin/mentor-dashboard
```

Remove the config
```bash
rm -f ~/.local/share/agent-mentor/sync.toml
```

# Sync
This plugin through the mcp saves topics and sessions in your local computer at this path ~/.local/share/agent-mentor.
If you want to re use it across different machines you can can copy and paste the entire content at that folder.

## Turso (optional)
You can optionally sync your knowledge database to [Turso](https://turso.tech) to keep it in sync across multiple machines automatically.

**Step 1 — Seed the remote database from your existing local file:**
```bash
turso db create mentor --from-file ~/.local/share/agent-mentor/knowledge.db
```

**Step 2 — Get your database URL and token from the Turso dashboard, then create a `sync.toml` file next to the knowledge database:**
```
~/.local/share/agent-mentor/sync.toml
```

```toml
[turso]
url = "libsql://your-database.turso.io"
token = "your-auth-token"
```

On next start the plugin will sync automatically with Turso.
