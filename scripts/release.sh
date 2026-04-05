#!/usr/bin/env bash
set -euo pipefail

CLAUDE_PLUGIN_JSON="claude-plugin/.claude-plugin/plugin.json"
CLAUDE_MARKETPLACE_JSON="claude-plugin/.claude-plugin/marketplace.json"
MCP_SCRIPT="scripts/mcp-server.sh"
DASHBOARD_SCRIPT="scripts/install-dashboard.sh"
MCP_CARGO_TOML="knowledge/Cargo.toml"
MCP_CARGO_TOML_LOCK="knowledge/Cargo.lock"

CURRENT_VERSION=$(grep '^VERSION=' "$MCP_SCRIPT" | sed 's/VERSION="v//;s/"//')

echo "Current version: ${CURRENT_VERSION}"
read -rp "New version: " VERSION

sed -i '' "s/^VERSION=\".*\"/VERSION=\"v${VERSION}\"/" "$MCP_SCRIPT"
sed -i '' "s/^version = \".*\"/version = \"${VERSION}\"/" "$MCP_CARGO_TOML"
sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$CLAUDE_PLUGIN_JSON"
sed -i '' "s/\"version\": \".*\"/\"version\": \"${VERSION}\"/" "$CLAUDE_MARKETPLACE_JSON"
sed -i '' "s/^VERSION=\".*\"/VERSION=\"v${VERSION}\"/" "$DASHBOARD_SCRIPT"

cd knowledge
cargo build
cd ..

git add "$CLAUDE_PLUGIN_JSON" "$CLAUDE_MARKETPLACE_JSON" "$MCP_SCRIPT" "$MCP_CARGO_TOML" "$MCP_CARGO_TOML_LOCK" "$DASHBOARD_SCRIPT"
git commit -m "release: v${VERSION}"
git tag "v${VERSION}"
git push origin main "v${VERSION}"
