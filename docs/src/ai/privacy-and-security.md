---
title: AI Privacy - Vela
description: Understand how Vela handles AI prompts, code context, hosted model requests, provider data boundaries, feedback, training data, and privacy controls.
---

# AI Privacy

This page explains the privacy and trust boundaries for AI features in Vela,
including [Vela Agent](./vela-agent.md), [Edit Prediction](./edit-prediction.md),
[Inline Assistant](./inline-assistant.md), and
[Git commit generation](../git.md#ai-support-in-git).

Vela does not retain your prompts or code context by default. For
[Vela-hosted models](../account/vela-hosted-models.md), Vela has no-training
commitments from model providers, and provider agreements require zero data
retention for inference requests except for
[provider-designated models with safety retention](#provider-safety-retention),
such as Anthropic's Covered Models.
Vela only retains AI data when you explicitly share feedback or opt in to
training data collection.

## AI Request Paths {#ai-request-paths}

| Path                                                         | Who handles model requests                        | What to know                                                                                                                                                                                                                   | Details                                                                                          |
| ------------------------------------------------------------ | ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------ |
| [Vela-hosted models](../account/vela-hosted-models.md)         | Vela routes requests to hosted model providers     | Provider agreements prohibit training on your prompts or code context and require zero data retention for inference requests, except for provider-designated models with safety retention, such as Anthropic's Covered Models. | [Vela-hosted model commitments](#data-retention-and-training)                                     |
| [Provider API keys](./use-api-access.md)                     | The configured provider                           | The provider handles requests under its own terms. Provider keys saved through Vela are stored in the system keychain, not in `settings.json`.                                                                                  | [Use API Access](./use-api-access.md)                                                            |
| [Existing subscriptions](./use-an-existing-subscription.md)  | The subscription provider                         | The provider handles requests under the subscription terms.                                                                                                                                                                    | [Use an Existing Subscription](./use-an-existing-subscription.md)                                |
| [Gateways](./use-a-gateway.md)                               | The configured gateway and upstream providers     | The gateway and upstream providers handle requests under their own terms.                                                                                                                                                      | [Use a Gateway](./use-a-gateway.md)                                                              |
| [Local models](./use-a-local-model.md)                       | The local server or self-hosted endpoint          | The local server handles requests according to how you configured that server.                                                                                                                                                 | [Use a Local Model](./use-a-local-model.md)                                                      |
| [External Agents](./external-agents.md)                      | The External Agent and its configured providers   | The External Agent handles model requests under its own terms. Tool and MCP behavior depends on agent and ACP configuration.                                                                                                   | [External Agents](./external-agents.md)                                                          |
| [Terminal Threads](./terminal-threads.md)                    | The CLI or TUI running in the terminal            | The CLI or TUI owns its auth, model routing, tools, instructions, MCP configuration, and data handling.                                                                                                                        | [Terminal Threads](./terminal-threads.md)                                                        |
| [Edit Prediction](./edit-prediction.md)                      | The selected edit prediction provider             | Each keystroke can send local editing context to the selected provider. Zeta requests are processed transiently unless training data collection is enabled; third-party providers follow their own terms.                      | [Edit Prediction](./edit-prediction.md), [Feedback and Training Data](./ai-improvement.md)       |
| [Agent tools](./tools.md), [MCP](./mcp.md), and integrations | Vela, configured MCP servers, and external systems | Tools can read, edit, search, run commands, fetch URLs, or call external systems depending on provider compatibility, project trust, MCP server, and tool permission settings.                                                 | [Tool Permissions](./tool-permissions.md), [MCP](./mcp.md)                                       |
| Project trust and instructions                               | Vela and the trusted worktree                      | Project-local instructions and skills are loaded from trusted worktrees. External Agents and Terminal Threads may read their own instruction files.                                                                            | [Worktree Trust](../worktree-trust.md), [Skills](./skills.md), [Instructions](./instructions.md) |

## Vela-Hosted Model Commitments {#data-retention-and-training}

For Vela-hosted models, Vela has commitments from model providers that prohibit
training on your prompts or code context and require zero data retention for
inference requests, except for
[provider-designated models with safety retention](#provider-safety-retention),
such as Anthropic's Covered Models. The public provider documents linked below describe provider programs or default
API terms; Vela-hosted model requests are governed by Vela's provider agreements.

| Provider  | No training reference                                   | Zero-data-retention reference                                                                                                                                                                      |
| --------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Anthropic | [Yes](https://www.anthropic.com/legal/commercial-terms) | [Yes](https://privacy.anthropic.com/en/articles/8956058-i-have-a-zero-data-retention-agreement-with-anthropic-what-products-does-it-apply-to), except [covered models](#provider-safety-retention) |
| Google    | [Yes](https://cloud.google.com/terms/service-terms)     | [Yes](https://cloud.google.com/terms/service-terms), see Service Terms sections 18 and 20(h)                                                                                                       |
| OpenAI    | [Yes](https://openai.com/enterprise-privacy/)           | [Yes](https://platform.openai.com/docs/guides/your-data)                                                                                                                                           |

### Provider Safety Retention for Designated Models {#provider-safety-retention}

Some providers require limited data retention for specific models as a condition
of offering them, on every platform where those models are available. Anthropic
retains prompts and outputs for models it designates as Covered Models, including
Claude Fable 5, for at least 30 days for trust and safety purposes. Vela cannot
opt out of this retention; it applies wherever these models are served. See
[Anthropic's data retention practices for Covered Models](https://support.claude.com/en/articles/15425996-data-retention-practices-for-covered-models).

For these models:

- The no-training commitment still applies. Retained data is used for safety
  review, not model training.
- Vela does not retain your prompts or outputs. Retention happens at the
  provider, under the provider's documented access controls and deletion
  timelines.
- All other Vela-hosted models keep zero-data-retention handling.

If you don't want provider-side retention, use a model that the provider has
not designated for safety retention. Switching to
[your own API key](./use-api-access.md) or
[subscription](./use-an-existing-subscription.md) does not avoid this retention
for covered models, because providers apply it on every platform where those
models are offered.

## AI Data Retained by Vela {#ai-data-retained-by-vela}

Vela may retain AI data only when you explicitly share it or opt in:

- [Response ratings and feedback](./ai-improvement.md#ai-feedback-with-ratings)
  can send a conversation thread to Vela for review and improvement.
- [Edit Prediction training data](./ai-improvement.md#edit-predictions) is
  collected only when you opt in, the project is open source, and the file is not
  excluded.

See [Feedback and Training Data](./ai-improvement.md) for the full list of what
can be stored in each opt-in case.

## Controls and Related Privacy Docs {#controls-and-related-privacy-docs}

- [Telemetry](../telemetry.md): What telemetry Vela collects and how to control
  it.
- [Privacy for Business](../business/privacy.md): How Vela Business enforces
  privacy settings across an organization.
- [Admin Controls](../business/admin-controls.md): How owners and admins control
  Vela-hosted models, Edit Prediction, and feedback sharing.
- [AI Quick Start](./quick-start.md#turn-ai-off): How to turn AI off.
- [Privacy Policy](https://vela.dev/privacy-policy): Vela's privacy policy.
- [Subprocessors](https://vela.dev/subprocessors): Vela's subprocessors.
- [Terms of Service](https://vela.dev/terms): Vela's terms.
