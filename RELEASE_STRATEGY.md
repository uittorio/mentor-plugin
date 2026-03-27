# Release strategy
This projects comes up with different tools for the agentic workflow. At the moment it provides

- Skills
- MCP
- configuration based on the coding agent

The skills are simple md files and the MCP is an application written in Rust. In order to prevent users the need to run rust in their agentic environment the release process will create a binary for different operating systems. 

When copying or installing this plugin we want to just provide the essential files instead of cloning the entire repo.

For Claude code plugin you can create a marketplace and easil install a plugin but that ends up cloning the entire repository.

To prevent that the release pipeline will push to separate branches the plugin files needed for each coding agent

- opencode-plugin-release
- clade-plugin-release

## How are the different files handled

### Skills 
Skills are purely copied in the release branch

### Configuration files
Configuration files are purely copied in the release branch. Examples
- opencode.json
- .mcp.json (claude code)
- 

### MCP binary
The mcp binary is released in the github release. For example, at this link you can find the [binaries] (https://github.com/uittorio/mentor-plugin/releases). Binaries dont need to be download because they are downloaded as part of the mcp-server.sh script that will download the correct binary based on your operating system. The file will be downloaded in this folder with a version to handle updates.

`${HOME}/.local/bin/mentor-mcp-${VERSION}`
