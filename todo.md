first part. MCP up and running

learn about mcp, how are theu called with a coding agent (you will see these in the agent visualiser)

Installation/release

think about folder structure
  we need a script to install the binary from releases for the mcp server and test it
  consider the fact that there is a claude plugin (we can try different folders)
  consider that some files needs to be excluded (rust source) (same as above)
  consider that skills can be shared (need to see if we can move them around)
  consider that hooks cannot be shared (need to see if we can move them around)
  readme installation is claude specific  

ci pipeline

install.sh file 
 - where the binary should live (it will be next to the plugin, should be okay)

uninstall.sh file
- we might not need this as evrything is inside the claude folder (for claude)


Add database
- decide where data lives
