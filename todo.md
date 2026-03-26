first part. MCP up and running

learn about mcp, how are theu called with a coding agent (you will see these in the agent visualiser)

Installation/release

Claude
  split branches for releases to keep opencode vs claudecode difference and lean. 
  generate binary in releases
  we need a script to install the binary from releases for the mcp server and test it
  consider that skills can be shared (need to see if we can move them around)

Opencode
  release for claude
  test the flow
  
Modify readme to include both claude and opencode

install

install.sh file 
 - where the binary should live (it will be next to the plugin, should be okay)

uninstall.sh file
- we might not need this as evrything is inside the claude folder (for claude)

start building the database part

Add database
- decide where data lives
