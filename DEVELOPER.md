# Developer

---

## Dependencies

- [Rust](https://rustup.rs/) (stable toolchain)

## Running tests

From the `knowledge/` directory:

```sh
# Unit and integration tests
cargo test --lib --bins

# E2e tests (requires a release build first)
cargo build --release
cargo test --test e2e
```

## Git hooks

A pre-commit hook is provided that runs the full test suite before every commit: unit tests, release build, and e2e tests.

To enable it:

```sh
cp githooks/pre-commit .git/hooks/pre-commit
chmod +x .git/hooks/pre-commit
```

> **Note:** This step must be done manually after cloning. Git does not run hooks automatically on clone.

## Project structure

- `knowledge/` — Rust workspace: MCP server and dashboard binaries, learning domain logic
- `claude-plugin/` — Claude Code plugin (skill files)
- `opencode-plugin/` — Opencode plugin (skill files)
- `githooks/` — Shared git hooks
- `scripts/` — Release and utility scripts


## Testing strategy

This project ships different components to be used with coding agent or just a simple binaries. It also contains business logic in the main components to make decisions about topic etc. 
We have various tests to help ourself during the development. 

### Installation tests
The installation of the plugin is mostly manual, installation scripts with some bash. To avoid breaking the release process and these installation scripts we have two install tests. These are useful to ensure that the binaries are installed correctly in a machine and the binary runs correctly. 
- [open code install](./tests_installs/opencode_install_tests.sh)
- [dashboard](./tests_installs/dashboard_install_tests.sh)


### Knowledge Mcp 
Mcp integration is used by the coding agent. To verify a full integration we should run some coding agents that use all the tools. There are some e2e tests of the mcp that go through the full journey. This allows us to ensure that the mcp connection works correctly and the tools are working correctly. Locally these tests use the debug build; in CI they run against the release binary via `BINARY_PATH`.

### Database integration
sql queries could easily break. While the MCP e2e tests cover us from high level connection and could cover database connection, these tests give us fast feedback and can check easily more path that would be difficult to verify in the e2e tests.

### Business logic
We currently test spaced repetition algorithm (sm2) and trigram similarity algorithm.
