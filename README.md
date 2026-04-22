# Feature Hub

Feature Hub is a desktop application for managing work features and epics with linked Claude Code sessions, external links, files, tasks, and notes. It ships a CLI (`fh`) and an MCP server (`fh-mcp`) for interacting with features from the terminal or directly from Claude Code sessions.

![Feature Hub](docs/screenshot.png)

## Key Features

- **Feature Management** -- organize work into features with statuses, tags, parent-child hierarchy, and groups
- **Claude Code Integration** -- built-in MCP server that gives Claude full context about your features, tasks, and notes
- **CLI Tool** -- terminal-based feature management via the `fh` command, with session start/resume support
- **Tasks and Notes** -- track tasks (with Jira sync support) and maintain rich markdown notes per feature
- **External Links** -- attach and auto-categorize links to GitHub, Jira, Figma, Confluence, Slack, and more
- **File Attachments** -- upload and organize files per feature with preview support
- **Knowledge Base** -- storage-scoped repository for HOW-TOs, findings, and research results accessible to Claude
- **Extension System** -- pluggable extensions for third-party integrations
- **Multi-Storage Support** -- maintain separate storage directories for different projects or teams

## Installation

Download the latest installer for your platform from the [Releases](https://github.com/LittleBrushGames/FeatureHub/releases) page.

Alternatively, build from source using the instructions below.

## Building from Source

### Prerequisites

- Node.js 18+
- Rust 1.75+
- npm

### Steps

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build
```

The production build outputs platform-specific installers in `src-tauri/target/release/bundle/`.

## CLI

The `fh` CLI provides terminal-based feature management. Use it to start and resume Claude Code sessions linked to features, list features, and more.

```bash
# Build and install the CLI
npm run build:cli
```

Run `fh --help` for available commands.

## Tech Stack

- **Desktop Shell** -- Tauri 2 (Rust backend)
- **Frontend** -- Svelte 5 + TypeScript + Vite
- **Storage** -- SQLite via rusqlite (bundled)
- **Styling** -- CSS custom properties (dark theme) + Tailwind v4
- **MCP Server** -- rmcp crate (stdio-based)

## License

This project is licensed under the MIT License. See [LICENSE](LICENSE) for details.
