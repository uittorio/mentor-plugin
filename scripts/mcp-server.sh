#!/usr/bin/env bash
# VERSION is set locally by scripts/release.sh before tagging.
VERSION="v0.1.0"

BINARY="${HOME}/.local/bin/mentor-mcp-${VERSION}"

if [ ! -f "$BINARY" ]; then
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)

  case "${OS}-${ARCH}" in
    linux-x86_64)            ARTIFACT="mentor-mcp-linux-x86_64" ;;
    darwin-arm64|darwin-aarch64) ARTIFACT="mentor-mcp-darwin-arm64" ;;
    darwin-x86_64)           ARTIFACT="mentor-mcp-darwin-x86_64" ;;
    *)
      echo "Unsupported platform: ${OS}-${ARCH}" >&2
      exit 1
      ;;
  esac

  mkdir -p "${HOME}/.local/bin"
  echo "Downloading mentor-mcp ${VERSION} for ${OS}-${ARCH}..." >&2
  curl -fsSL \
    "https://github.com/uittorio/mentor-plugin/releases/download/${VERSION}/${ARTIFACT}" \
    -o "$BINARY"
  chmod +x "$BINARY"
fi

exec "$BINARY" "$@"
