#!/usr/bin/env bash
# VERSION is set locally by scripts/release.sh before tagging.
VERSION="0.0.25"
REPO="uittorio/mentor-plugin"

BINARY="${HOME}/.local/bin/mentor-dashboard"

# Supported artifacts — add new platforms here
ARTIFACT_linux_x86_64="mentor-dashboard-linux-x86_64"
ARTIFACT_darwin_arm64="mentor-dashboard-darwin-arm64"

if [ -f "$BINARY" ] && [ "$($BINARY --version)" = "$VERSION" ]; then
    echo "Already up to date" >&2
else
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
  echo "Downloading mentor-dashboard ${VERSION} for ${OS}-${ARCH}..." >&2
  curl -fsSL \
    "https://github.com/${REPO}/releases/download/v${VERSION}/${ARTIFACT}" \
    -o "$BINARY"
  chmod +x "$BINARY"
fi

echo "ensure ~/.local/bin is on your PATH" >&2
