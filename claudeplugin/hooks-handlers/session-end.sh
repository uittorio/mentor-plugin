#!/usr/bin/env bash
# Mentor plugin — SessionEnd hook
# Moves the in-progress learning recap draft to the Obsidian vault.
# Runs on every session exit (graceful or not).

set -euo pipefail

PROJECT_DIR="${CLAUDE_PROJECT_DIR:-$PWD}"
CONFIG="$PROJECT_DIR/.claude/mentor-config.json"
DRAFT="$PROJECT_DIR/.claude/mentor-session-draft.md"

# Nothing to do if no draft was written this session
[[ ! -f "$DRAFT" ]] && exit 0

# Nothing to do if vault was never configured
[[ ! -f "$CONFIG" ]] && exit 0

VAULT_PATH=$(python3 -c "import json; d=json.load(open('$CONFIG')); print(d.get('vault_path',''))" 2>/dev/null || true)
TOPIC=$(python3 -c "import json; d=json.load(open('$CONFIG')); print(d.get('topic','session'))" 2>/dev/null || echo "session")
RECAP_DIR=$(python3 -c "import json; d=json.load(open('$CONFIG')); print(d.get('recap_dir',''))" 2>/dev/null || true)

[[ -z "$VAULT_PATH" ]] && exit 0

# Fall back to a sensible default if recap_dir was never written (old config)
[[ -z "$RECAP_DIR" ]] && RECAP_DIR="$VAULT_PATH/recaps/$TOPIC"

# Read the exit reason from stdin (JSON from Claude Code)
INPUT=$(cat)
REASON=$(echo "$INPUT" | python3 -c "
import sys, json
try:
    d = json.load(sys.stdin)
    print(d.get('session_end_reason', 'unknown'))
except Exception:
    print('unknown')
" 2>/dev/null || echo "unknown")

# If the session ended without an explicit close, flag it in the draft
if [[ "$REASON" != "prompt_input_exit" && "$REASON" != "logout" ]]; then
  printf '\n---\n> **Note:** Session ended unexpectedly (`%s`). Recap may be incomplete — review before next session.\n' "$REASON" >> "$DRAFT"
fi

# Ensure destination folder exists
DEST_DIR="$RECAP_DIR"
mkdir -p "$DEST_DIR"

DATE=$(date +%Y-%m-%d)
DEST_FILE="$DEST_DIR/$DATE $TOPIC.md"

# If a file for today already exists, append with a separator; otherwise move fresh
if [[ -f "$DEST_FILE" ]]; then
  printf '\n---\n\n' >> "$DEST_FILE"
  cat "$DRAFT" >> "$DEST_FILE"
  rm -f "$DRAFT"
else
  mv "$DRAFT" "$DEST_FILE"
fi

exit 0
