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

# Uninstall

Remove the mentor dashboard binary:
```bash
rm -f ~/.local/bin/mentor-dashboard
```
