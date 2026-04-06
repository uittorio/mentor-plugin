#!/usr/bin/env bash

set -euo pipefail

curl -fsSL https://raw.githubusercontent.com/uittorio/mentor-plugin/main/scripts/install-dashboard.sh | bash

VERSION="$(echo "$GITHUB_REF" | sed 's/refs\/tags\/v//')"

BINARY="${HOME}/.local/bin/mentor-dashboard"

BINARY_OUTPUT="$($BINARY --version)"

if [ "$BINARY_OUTPUT" = "$VERSION" ]; then
    echo "Version match. Installation successful: version ${VERSION}"
else
    echo "Version mismatch. Installation failed: binary output: ${BINARY_OUTPUT}. version ${VERSION}"
    exit 1
fi
