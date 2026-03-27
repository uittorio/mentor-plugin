# Installation

Releases are distributed via dedicated branches — see [Release Strategy](./RELEASE_STRATEGY.md) for details.

---

## OpenCode

```bash
curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/opencode-plugin-release/install.sh | bash
```

The installer will ask whether to install globally (`~/.config/opencode/`) or for the current project only (`.opencode/`). The MCP binary is downloaded automatically on first use.

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

## All coding agents

Remove the downloaded binary:
```bash
rm -f ~/.local/bin/mentor-mcp-*
```

## OpenCode

Remove the skills from wherever you installed them (global or project):
```bash
# global
rm -rf ~/.config/opencode/skills/mentor ~/.config/opencode/skills/mentor+

# project
rm -rf .opencode/skills/mentor .opencode/skills/mentor+
```

Remove the MCP server script:
```bash
rm -rf ~/.config/opencode/agent-mentor
```

Then remove the `agent-mentor` entry from your `opencode.json` (global or project).

## Claude Code

```
/plugin uninstall mentor-plugin
```
