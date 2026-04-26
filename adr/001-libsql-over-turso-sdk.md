# ADR 001: Use libsql instead of the turso SDK for local database access

**Date:** 2026-04-26

## Context

The mentor-plugin ships two binaries that both read the same local SQLite database:

- `knowledge-mcp` — the MCP server, invoked by the coding agent
- `knowledge-dashboard` — the TUI dashboard, run manually by the user

Both processes can be alive at the same time. The database is a local file managed by libsql with optional sync to a remote Turso replica.

The `turso` SDK (the newer first-party client library from Turso) was considered as a replacement for `libsql`. It provides a cleaner API and improved CDC-based sync with the remote replica.

## Decision

Stay on `libsql` rather than migrating to the `turso` SDK.

## Reason

The `turso` SDK does not allow more than one process to open the same local database file simultaneously. Since the MCP server and the dashboard can run concurrently, this would cause one of them to fail to acquire the database, breaking the user's workflow.

`libsql` uses standard SQLite locking semantics, which allows multiple readers on the same file without conflict.

## Consequences

- We remain on `libsql` until the `turso` SDK supports multi-process read access, or until the architecture changes (e.g. the two binaries talk to a single local server process instead of opening the file directly).
- CDC-based sync improvements in the `turso` SDK are not available for now.
