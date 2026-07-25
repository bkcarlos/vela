---
title: AI Quick Start - Vela
description: Choose the right Vela AI setup path for agents, models, subscriptions, local models, edit prediction, and privacy.
---

# AI Quick Start

Use this page to choose the right AI setup path in Vela. If you already know the company, subscription, model provider, agent, or CLI you want to use, start with [AI by Company](./by-company.md).

## Use the Vela Agent {#vela-agent}

Use [Vela Agent](./vela-agent.md) when you want Vela's native agent to read, edit, search, and run code in your project.

Vela Agent uses Vela-configured models from [LLM Providers](./llm-providers.md). It also uses Vela's built-in tools, [Tool Permissions](./tool-permissions.md), [Skills](./skills.md), [Instructions](./instructions.md), and [MCP servers](./mcp.md).

Start in the [Agent Panel](./agent-panel.md) to prompt the agent, add context, review changes, and manage threads.

### Configure the Vela Agent {#configure-vela-agent}

| If you want to...                         | Use                                       |
| ----------------------------------------- | ----------------------------------------- |
| Choose Manual, Auto, or Full Access       | [Tool Permissions](./tool-permissions.md) |
| Approve or deny individual tools          | [Tool Permissions](./tool-permissions.md) |
| Add reusable task instructions            | [Skills](./skills.md)                     |
| Add always-on personal or project context | [Instructions](./instructions.md)         |
| Connect external tools and context        | [Model Context Protocol](./mcp.md)        |

## Use Another Coding Agent in Vela {#agent-cli}

Use this path for Claude, Codex, OpenCode, Copilot, Cursor, Pi Coding Agent, Gemini CLI, or another coding agent.

| If the agent...                 | Use                                       |
| ------------------------------- | ----------------------------------------- |
| Integrates with Vela through ACP | [External Agents](./external-agents.md)   |
| Runs as a CLI or TUI            | [Terminal Threads](./terminal-threads.md) |

External Agents and Terminal Threads usually own their own auth, model configuration, subscriptions, tools, instructions, and MCP configuration.

## Choose Which Models the Vela Agent Uses {#model-access}

The Vela Agent and other model-backed Vela AI features use models configured through [LLM Providers](./llm-providers.md).

| If you want to...                                                             | Use                                                               |
| ----------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| Use models billed through Vela                                                 | [Vela-Hosted Models](../account/vela-hosted-models.md)              |
| Bring your own provider API key, credits, top-ups, or usage billing           | [Use API Access](./use-api-access.md)                             |
| Use a subscription you already pay for                                        | [Use an Existing Subscription](./use-an-existing-subscription.md) |
| Use OpenRouter, Vercel AI Gateway, Amazon Bedrock, or another gateway         | [Use a Gateway](./use-a-gateway.md)                               |
| Use Ollama, LM Studio, local OpenAI-compatible servers, or self-hosted models | [Use a Local Model](./use-a-local-model.md)                       |

Provider keys saved through Vela are stored in the system keychain, not in `settings.json`.

## Change AI Settings {#ai-settings}

| If you want to...                                                           | Go to                                                                    |
| --------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| Configure LLM providers, External Agents, or MCP servers                    | [Agent Settings](./agent-settings.md) with {#action agent::OpenSettings} |
| Disable AI, configure tool permissions, or set up edit prediction providers | Settings Editor with {#action vela::OpenSettings}                         |
| Edit advanced JSON-only settings                                            | Settings file with {#action vela::OpenSettingsFile}                       |

For general settings mechanics, see [Configuring Vela](../configuring-vela.md).

## Use a Specific AI Feature {#features}

| If you want to...                              | Use                                                  |
| ---------------------------------------------- | ---------------------------------------------------- |
| Prompt agents, add context, and review changes | [Agent Panel](./agent-panel.md)                      |
| Accept AI completions while typing             | [Edit Prediction](./edit-prediction.md)              |
| Rewrite selected code or terminal text         | [Inline Assistant](./inline-assistant.md)            |
| Run multiple AI tasks at once                  | [Parallel Agents](./parallel-agents.md)              |
| Generate commit messages                       | [Git commit generation](../git.md#ai-support-in-git) |

## Learn More {#learn-more}

| If you want to...                    | Use                                                                                                                                            |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Understand privacy and data controls | [AI Privacy](./privacy-and-security.md) and [Feedback and Training Data](./ai-improvement.md)                                                  |
| Understand plans, usage, and billing | [Plans & Pricing](../account/plans-and-pricing.md), [Vela-Hosted Models](../account/vela-hosted-models.md), and [Billing](../account/billing.md) |

## Turn AI Off {#turn-ai-off}

Open the Settings Editor with {#action vela::OpenSettings}, search for `Disable AI`, and enable it.

You can also add this to your settings file:

```json [settings]
{
  "disable_ai": true
}
```

When AI is turned off, all AI features are disabled. This includes the Threads Sidebar, Agent Panel, Edit Prediction, and Inline Assistant.
