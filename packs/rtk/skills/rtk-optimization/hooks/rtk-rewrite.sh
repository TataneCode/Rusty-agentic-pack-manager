#!/usr/bin/env bash
# rtk-hook-version: 3 (Gemini CLI port, shipped by the rtk pack)
# RTK Gemini CLI hook — rewrites run_shell_command calls to use rtk for token savings.
# Packs have no hook slot: register it manually in ~/.gemini/settings.json,
#   BeforeTool / matcher "run_shell_command" /
#   command: bash "$GEMINI_PROJECT_DIR/.gemini/skills/rtk-optimization/hooks/rtk-rewrite.sh"
# Requires: rtk >= 0.23.0, jq
#
# Thin delegating hook: all rewrite logic lives in `rtk rewrite`, which is the
# single source of truth (src/discover/registry.rs in rtk-ai/rtk).
#
# `rtk rewrite` exit code protocol:
#   0 + stdout  Rewrite found, no deny/ask rule matched -> rewrite (allow)
#   1           No RTK equivalent -> pass through unchanged
#   2           Deny rule matched -> pass through (Gemini's own confirmation handles it)
#   3 + stdout  Ask rule matched -> rewrite, but let Gemini still prompt the user
#
# Gemini CLI BeforeTool output: `hookSpecificOutput.tool_input` merges with and
# overrides the model's arguments. Gemini has no auto-allow decision for
# BeforeTool, so every rewrite still goes through Gemini's normal confirmation;
# the hook only rewrites, never blocks.

if ! command -v jq &>/dev/null; then
  echo "[rtk] WARNING: jq is not installed. Hook cannot rewrite commands. Install: https://jqlang.github.io/jq/download/" >&2
  echo '{}'
  exit 0
fi

if ! command -v rtk &>/dev/null; then
  echo "[rtk] WARNING: rtk is not installed or not in PATH. Hook cannot rewrite commands. Install: https://github.com/rtk-ai/rtk#installation" >&2
  echo '{}'
  exit 0
fi

# Version guard: rtk rewrite was added in 0.23.0.
# Older binaries: warn once and pass through cleanly (no silent failure).
# Cache the version check to avoid spawning multiple processes on every hook call.
CACHE_DIR=${XDG_CACHE_HOME:-$HOME/.cache}
CACHE_FILE="$CACHE_DIR/rtk-hook-version-ok"
if [ ! -f "$CACHE_FILE" ]; then
  RTK_VERSION_RAW=$(rtk --version 2>/dev/null)
  RTK_VERSION=${RTK_VERSION_RAW#rtk }
  RTK_VERSION=${RTK_VERSION%% *}
  if [ -n "$RTK_VERSION" ]; then
    IFS=. read -r MAJOR MINOR PATCH <<<"$RTK_VERSION"
    # Require >= 0.23.0
    if [ "$MAJOR" -eq 0 ] && [ "$MINOR" -lt 23 ]; then
      echo "[rtk] WARNING: rtk $RTK_VERSION is too old (need >= 0.23.0). Upgrade: cargo install --git https://github.com/rtk-ai/rtk rtk" >&2
      echo '{}'
      exit 0
    fi
  fi
  mkdir -p "$CACHE_DIR" 2>/dev/null
  touch "$CACHE_FILE" 2>/dev/null
fi

INPUT=$(cat)
CMD=$(jq -r '.tool_input.command // empty' <<<"$INPUT")

if [ -z "$CMD" ]; then
  echo '{}'
  exit 0
fi

# Delegate all rewrite + permission logic to the Rust binary.
REWRITTEN=$(rtk rewrite "$CMD" 2>/dev/null)
EXIT_CODE=$?

case $EXIT_CODE in
  0)
    # Rewrite found, no permission rules matched.
    # If the output is identical, the command was already using RTK.
    [ "$CMD" = "$REWRITTEN" ] && { echo '{}'; exit 0; }
    ;;
  1)
    # No RTK equivalent — pass through unchanged.
    echo '{}'
    exit 0
    ;;
  2)
    # Deny rule matched — let Gemini CLI's native confirmation handle it.
    echo '{}'
    exit 0
    ;;
  3)
    # Ask rule matched — rewrite, but do not auto-allow so Gemini prompts.
    ;;
  *)
    echo '{}'
    exit 0
    ;;
esac

jq -c --arg cmd "$REWRITTEN" \
  '{
    "hookSpecificOutput": {
      "hookEventName": "BeforeTool",
      "tool_input": { "command": $cmd }
    }
  }' <<<"$INPUT"
