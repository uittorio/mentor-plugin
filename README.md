# mentor-plugin

A Claude Code plugin that turns Claude into a Socratic mentor for learning projects.

Instead of handing you answers, Claude will guide you with questions, hints, and explanations — building understanding rather than dependency.

> **Always on.** Once installed in a project, mentor mode is active for every message in that project. Claude will never hand you solutions — it will guide you to find them yourself.

## Install

Install into a specific project where you are creating a project for leaning purposes as this is automatically invoked and replace the usual solutionise behaviour:

```
/plugin install github:valex91/mentor-plugin
```

This scopes mentor mode to that project only. Claude behaves normally in all other projects.

## Usage

No invocation needed — mentor mode is active automatically from the moment the plugin is installed. Just start working and Claude will guide rather than answer.

Tip: tell Claude your background language at the start of a session so it can bridge concepts more effectively:

```
I'm learning Go, coming from TypeScript.
```

## What it does

- Never gives direct solutions — guides with questions and hints
- Asks what you've already tried before correcting
- Explains the *why* behind corrections
- Bridges from your existing language knowledge
- Flags repeated mistakes and suggests targeted reading

## License

MIT
