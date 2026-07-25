---
title: Vela Agent
description: Use Vela's native AI agent with Vela-configured models, tools, profiles, skills, instructions, and MCP servers.
---

# Vela Agent

Vela Agent is Vela's native agent path. It runs in the [Agent Panel](./agent-panel.md) and [Threads Sidebar](./parallel-agents.md#threads-sidebar), uses models configured through [LLM Providers](./llm-providers.md), and integrates with Vela's project, editor, terminal, and review surfaces.

Use Vela Agent when you want the agent to:

- read and search your project
- edit files
- run terminal commands
- use Vela-managed MCP tools
- follow [Tool Permissions](./tool-permissions.md)
- use Vela [Skills](./skills.md) and [Instructions](./instructions.md)
- show changes in Vela's review UI

## What Vela Agent Uses {#what-vela-agent-uses}

| Capability                 | Source of truth                           |
| -------------------------- | ----------------------------------------- |
| Model access               | [LLM Providers](./llm-providers.md)       |
| Panel workflow             | [Agent Panel](./agent-panel.md)           |
| Tool approval behavior     | [Tool Permissions](./tool-permissions.md) |
| Built-in tools             | [Tools](./tools.md)                       |
| External tools             | [MCP](./mcp.md)                           |
| Reusable task instructions | [Skills](./skills.md)                     |
| Always-on instructions     | [Instructions](./instructions.md)         |

## How It Differs from Other Agent Paths {#other-agent-paths}

| Agent path                                | Main difference                                                                              |
| ----------------------------------------- | -------------------------------------------------------------------------------------------- |
| [Vela Agent](./vela-agent.md)               | Uses Vela's model, tool, profile, skill, instruction, and MCP configuration                   |
| [External Agents](./external-agents.md)   | Use an ACP integration and often own auth, model, tool, and native instruction configuration |
| [Terminal Threads](./terminal-threads.md) | Run a CLI/TUI in a terminal-backed thread; the CLI owns auth and configuration               |

See [Agents](./agents.md) for the full comparison.
