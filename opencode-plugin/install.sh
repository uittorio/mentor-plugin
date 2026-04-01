#!/usr/bin/env bash
set -euo pipefail

REPO="uittorio/mentor-plugin"
BRANCH="opencode-plugin-release"
BASE_URL="https://raw.githubusercontent.com/${REPO}/${BRANCH}"

MODE="${1:-}"

download() {
  local src="$1" dest="$2"
  curl -fsSL "${BASE_URL}/${src}" -o "$dest"
}

# ── Scope ────────────────────────────────────────────────────────────────────
echo ""
echo "Mentor Plugin — OpenCode Installer"
echo ""

SKILLS_DEST="${HOME}/.config/opencode/skills"
CONFIG_FILE="${HOME}/.config/opencode/opencode.json" ;

# TODO re think about local config vs global config
# Local config is currently deactivated because this process copies the skill files in your project and it is not ideal to maintain
#
# if [ "$MODE" = "global" ]; then
#   SKILLS_DEST="${HOME}/.config/opencode/skills"
#   CONFIG_FILE="${HOME}/.config/opencode/opencode.json" ;
# else
#   SKILLS_DEST="$(pwd)/.opencode/skills"
#   CONFIG_FILE="$(pwd)/opencode.json"
# fi

MCP_DEST="${HOME}/.config/opencode/agent-mentor/mcp-server.sh"

# ── Skills ───────────────────────────────────────────────────────────────────
echo ""
mkdir -p "${SKILLS_DEST}/mentor+"
download "skills/mentor+/SKILL.md" "${SKILLS_DEST}/mentor+/SKILL.md"
echo "✓ Skills installed → ${SKILLS_DEST}"

# ── MCP server script ────────────────────────────────────────────────────────
mkdir -p "$(dirname "$MCP_DEST")"
download "agent-mentor/mcp-server.sh" "$MCP_DEST"
chmod +x "$MCP_DEST"
echo "✓ MCP server script → ${MCP_DEST}"

# ── opencode.json ────────────────────────────────────────────────────────────
MCP_SNIPPET=$(cat <<EOF
  "mcp": {
    "agent-mentor": {
      "type": "local",
      "command": ["${MCP_DEST}"]
    }
  }
EOF
)

if [ ! -f "$CONFIG_FILE" ]; then
  cat > "$CONFIG_FILE" <<EOF
{
  "\$schema": "https://opencode.ai/config.json",
${MCP_SNIPPET}
}
EOF
  echo "✓ Config created → ${CONFIG_FILE}"
elif grep -q '"agent-mentor"' "$CONFIG_FILE"; then
  echo "✓ agent-mentor already in ${CONFIG_FILE} — skipping"
else
  echo ""
  echo "Add this to your ${CONFIG_FILE}:"
  echo ""
  echo "$MCP_SNIPPET"
fi

echo ""
echo "Done."
