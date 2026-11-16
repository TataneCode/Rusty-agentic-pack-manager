# graphify — Gemini CLI: native support, nothing to convert

Source: [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify)

The README of the source repo documents first-class Gemini CLI support, and the
repo itself ships a ready-made extension manifest (`gemini-extension.json` with
`mcpServers.graphify` + `contextFileName: GEMINI.md`). Per the conversion
brief: no port needed — the official install path is enough.

## Install

```bash
# 1. Install the graphify CLI (with the MCP server extra)
uv tool install "graphifyy[mcp]"
# add the [gemini] extra if you want LLM extraction via the Gemini API:
# uv tool install "graphifyy[mcp,gemini]"

# 2. Install the Gemini CLI integration (writes the extension + skill)
graphify install --platform gemini
```

See the upstream README install table: `Gemini CLI | graphify install --platform gemini`.

## What you get

- MCP server `graphify` (graph queries: callers, dependencies, change impact, tests)
  exposed to Gemini CLI via the extension's `mcpServers`.
- `GEMINI.md` context + `/graphify` skill installed by `graphify install`.
- Optional hook nudges toward the graph before search-style tool calls
  (Gemini CLI hook platforms are supported upstream).
