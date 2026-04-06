# Developer

---

# Testing strategy

This project ships different components to be used with coding agent or just a simple binaries. It also contains business logic in the main components to make decisions about topic etc. 
We have various tests to help ourself during the development. 

### Installation tests
The installation of the plugin is mostly manual, installation scripts with some bash. To avoid breaking the release process and these installation scripts we have two install tests. These are useful to ensure that the binaries are installed correctly in a machine and the binary runs correctly. 
- [open code install](./tests-installs/opencode_install_tests.sh)
- [dashboard](./tests-installs/dashboard_install_tests.sh)


### Knowledge Mcp 
Mcp integration is used by the coding agent. To verify a full integration we should run some coding agents that use all the tools. There are some e2e tests of the mcp that go through the full journey. This allows us to ensure that the mcp connection works correctly and the tools are working correctly. These tests uses the binary released.

### Database integration
Queries could be easily broken. While the MCP e2e tests cover us from high level connection and could cover database connection, these tests give us fast feedback and can check easily more path that would be difficult to verify in the e2e tests.

### Business logic
We currently test spaced repetition algorithm (sm2) and trigram similarity algorithm.
