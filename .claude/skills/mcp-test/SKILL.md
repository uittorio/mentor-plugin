---
name: mcp-test
description: Quick smoke test for the agent-mentor-local MCP server. Calls all three tools and prints the raw responses.
version: 0.1.0
user-invocable: true
---

# MCP Smoke Test

Call all three tools from `agent-mentor` in sequence and print each raw response. Do not interpret or act on the results — just show them.

## Steps

1. Call `get_review_candidates` with `limit: 3`. Print the raw result.
2. Call `knowledge_check` with `topics: ["Rust ownership", "JSON-RPC"]`. Print the raw result.
3. Call `update_knowledge` with `topic: "Rust ownership"`, `quality: 3`. Print the raw result.

After all three, print a one-line summary: which tools succeeded and which failed.

$ARGUMENTS
