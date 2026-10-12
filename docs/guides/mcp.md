---
id: mcp
title: Model Context Protocol (MCP) Integration & Setup
aliases: [mcp-server, agent-setup, claude-desktop, cursor]
summary: How to register kinema as a Model Context Protocol (MCP) server for AI coding agents.
category: guide
---

# Model Context Protocol (MCP) Integration

`kinema` can run as an offline Model Context Protocol (MCP) stdio server, allowing AI coding assistants (such as Claude Desktop, Claude Code, Cursor, and other MCP-compatible clients) to directly invoke kinema verification tools and read architecture documentation.

## Running the Server

Launch the MCP server in your project root:

```bash
kinema mcp
```

The server communicates using JSON-RPC over standard input/output (`stdio`).

## Registering with MCP Clients

### Claude Code CLI

Add `kinema` to your project or global MCP configuration:

```bash
claude mcp add kinema -- kinema mcp
```

### Claude Desktop

Edit your Claude Desktop configuration file:
- **macOS**: `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows**: `%APPDATA%\Claude\claude_desktop_config.json`
- **Linux**: `~/.config/Claude/claude_desktop_config.json`

Add the `kinema` server entry under `mcpServers`:

```json
{
  "mcpServers": {
    "kinema": {
      "command": "kinema",
      "args": ["mcp"]
    }
  }
}
```

## Available Capabilities

When registered as an MCP server, kinema provides:

1. **Integrated Verification Tools**:
   - `fmt`: Canonical source formatting
   - `check`: Multi-stage verification (`parse`, `fmt`, `lint`, `equiv`, `drc`)
   - `ir`: Flat netlist intermediate representation
   - `netlist`: KiCad `.net` netlist generation
   - `rules`: `.kicad_pro` and `.kicad_dru` rules generation
   - `graph`: Netlistsvg-compatible JSON visualization
   - `gen-leaf`: Scaffold component leaf modules

2. **Documentation Tools & Resources**:
   - `doc_list`: Discover guides, rules, and best practices
   - `doc_read`: Read targeted documentation sections and remediations

See also:
- [End-to-End PCB Design & Verification Workflow](workflow.md)
- [Core Design Principles & Operating Guidelines](principles.md)
