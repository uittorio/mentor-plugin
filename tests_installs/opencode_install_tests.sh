#!/usr/bin/env bash

set -euo pipefail

curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/opencode-plugin-release/install.sh | bash

VERSION="$(echo "$GITHUB_REF" | sed 's/refs\/tags\/v//')"

MCP_BINARY="${HOME}/.config/opencode/agent-mentor/mcp-server.sh"

MCP_BINARY_OUTPUT="$($MCP_BINARY --version)"

if [ "$MCP_BINARY_OUTPUT" = "$VERSION" ]; then
    echo "Version match. Installation successful"
else
    echo "Version mismatch. Installation failed"
    exit 1
fi
