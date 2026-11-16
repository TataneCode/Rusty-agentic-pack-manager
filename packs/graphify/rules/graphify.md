# graphify — native CLI support, nothing to port

Source: [Graphify-Labs/graphify](https://github.com/Graphify-Labs/graphify) (MIT).

The upstream repo documents first-class support for both target CLIs of this
pack and ships its own Gemini CLI extension manifest (`gemini-extension.json`
with `mcpServers.graphify` + `GEMINI.md`), so there is no component to convert
— the official install path is enough. This rule only records how to install.

## Install

```bash
# 1. Install the graphify CLI (with the MCP server extra)
uv tool install "graphifyy[mcp]"
# add the [gemini] extra if you want LLM extraction via the Gemini API:
# uv tool install "graphifyy[mcp,gemini]"

# 2a. Gemini CLI: native platform install (writes the extension + /graphify skill)
graphify install --platform gemini

# 2b. Vibe CLI: graphify has no dedicated vibe platform, use the generic
#     Agent Skills install — Vibe CLI discovers skills in ~/.agents/skills/
graphify install --platform agents
```

See the upstream README install table: `Gemini CLI | graphify install --platform gemini`.

## What you get

- MCP server `graphify` (graph queries: callers, dependencies, change impact,
  tests) exposed to Gemini CLI via the extension's `mcpServers`.
- `GEMINI.md` context + `/graphify` skill installed by `graphify install`.
- Optional hook nudges toward the graph before search-style tool calls
  (Gemini CLI hook platforms are supported upstream).
