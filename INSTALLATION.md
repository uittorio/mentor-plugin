# Installation

This repository is structured in such a way that the releases are in specific branches. You can find more info [here](./RELEASE_STRATEGY.md)

## Opencode

Navigate to https://github.com/uittorio/mentor-plugin/tree/opencode-plugin-release

Download the content and copy it in your local project or in your project config (.opencode) or in the global opencode config (~/.config/opencode) [opencode](https://opencode.ai/docs/skills/#place-files)

When starting the mcp the first time it will download the binary from the github [releases](https://github.com/uittorio/mentor-plugin/releases).


## Claude code
**Step 1 — Add the marketplace:**
```
/plugin marketplace add uittorio/mentor-plugin@claude-plugin-release
```

**Step 2 — Install the plugin:**
```
/plugin install mentor-plugin@mentor-plugins
```

# Updates

## Opencode
Follow the installation instructions

## Claude code
Claude code handles updates automatically internally, you can just type /marketplace, find the marketplace and update the plugin.

# Uninstall

## Opencode

## Claude code
