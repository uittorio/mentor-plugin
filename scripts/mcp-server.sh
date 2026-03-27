#!/usr/bin/env bash
# VERSION is set locally by scripts/release.sh before tagging.
VERSION="v0.0.15"
REPO="uittorio/mentor-plugin"

# Supported artifacts — add new platforms here
ARTIFACT_linux_x86_64="mentor-mcp-linux-x86_64"
ARTIFACT_darwin_arm64="mentor-mcp-darwin-arm64"

BINARY="${HOME}/.local/bin/mentor-mcp-${VERSION}"

if [ ! -f "$BINARY" ]; then
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  # Normalise aarch64 → arm64
  [[ "$ARCH" == "aarch64" ]] && ARCH="arm64"

  VAR="ARTIFACT_${OS}_${ARCH}"
  ARTIFACT="${!VAR:-}"

  if [ -z "$ARTIFACT" ]; then
    echo "Unsupported platform: ${OS}-${ARCH}" >&2
    exit 1
  fi

  mkdir -p "${HOME}/.local/bin"
  echo "Downloading mentor-mcp ${VERSION} for ${OS}-${ARCH}..." >&2
  curl -fsSL \
    "https://github.com/${REPO}/releases/download/${VERSION}/${ARTIFACT}" \
    -o "$BINARY"
  chmod +x "$BINARY"
fi

exec "$BINARY" "$@"
