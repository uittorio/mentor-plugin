# Installation

Releases are distributed via dedicated branches — see [Release Strategy](./RELEASE_STRATEGY.md) for details.

---

## OpenCode
The install plugin will copy the nececessary files from the release branch in your opencode project. You can either install it globally or local to your project. I advise to install it in your current project first.

### Global config (`~/.config/opencode/`)
```bash
curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/opencode-plugin-release/install.sh | bash -s global
```

### Local config (`.opencode/`)
```bash
curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/opencode-plugin-release/install.sh | bash -s
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
rm -f ~/.local/bin/mentor-mcp-*
rm -rf ~/.local/share/agent-mentor
```

## OpenCode

Remove the skill:
```bash
# local
rm -rf .opencode/skills/mentor+

# global
rm -rf ~/.config/opencode/skills/mentor+
```

Remove the MCP server script:
```bash
rm -rf ~/.config/opencode/agent-mentor
```

Remove the `agent-mentor` entry from your `opencode.json` (local or global).

Remove the binary and knowledge database:
```bash
rm -f ~/.local/bin/mentor-mcp-*
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
| MCP binary (downloaded on first run) | `~/.local/bin/mentor-mcp-<version>` |
| Knowledge database (created on first run) | `~/.local/share/agent-mentor/knowledge.db` |

### OpenCode — local install

| File | Destination |
|------|-------------|
| `SKILL.md` | `.opencode/skills/mentor+/SKILL.md` |
| `mcp-server.sh` | `~/.config/opencode/agent-mentor/mcp-server.sh` |
| MCP config entry | added to `./opencode.json` |
| MCP binary (downloaded on first run) | `~/.local/bin/mentor-mcp-<version>` |
| Knowledge database (created on first run) | `~/.local/share/agent-mentor/knowledge.db` |

### OpenCode — global install

| File | Destination |
|------|-------------|
| `SKILL.md` | `~/.config/opencode/skills/mentor+/SKILL.md` |
| `mcp-server.sh` | `~/.config/opencode/agent-mentor/mcp-server.sh` |
| MCP config entry | added to `~/.config/opencode/opencode.json` |
| MCP binary (downloaded on first run) | `~/.local/bin/mentor-mcp-<version>` |
| Knowledge database (created on first run) | `~/.local/share/agent-mentor/knowledge.db` |
