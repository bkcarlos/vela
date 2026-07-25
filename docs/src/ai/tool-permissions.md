# Tool Permissions

Configure which [Agent Panel](./agent-panel.md) tools run automatically and which require your approval.
For a list of available tools, [see the Tools page](./tools.md).

Use `agent.permission_mode` to choose the overall permission mode.

## Permission Modes

```json [settings]
{
  "agent": {
    "permission_mode": "manual"
  }
}
```

`permission_mode` accepts the following values:

- **Manual** (`"manual"`, default): Requires confirmation before the agent modifies files or runs terminal commands. Explicit per-tool allow rules can still approve trusted actions automatically.
- **Auto** (`"auto"`): Automatically performs low-risk actions and asks for confirmation before high-risk actions.
- **Full Access** (`"full_access"`): Skips normal permission confirmations and disables the terminal sandbox.

All three modes expose the same provider-compatible built-in and MCP tools. The selected mode changes how tool actions are approved and isolated, not which capabilities the agent can see.

<div class="warning">

Full Access allows the agent to run arbitrary commands and modify files without confirmation, and terminal commands are not isolated by the sandbox.
Only use it in environments and repositories you trust.

</div>

### Auto Risk Categories

Auto mode allows routine edits and commands, then applies an additional safety check before execution.
It asks for confirmation for actions such as:

- Destructive Git commands, including force pushes, hard resets, forced cleans, branch deletion, and clearing stashes
- Recursive deletion and deleting paths through the file tools
- Deployments, infrastructure changes, IAM changes, and database migrations
- Downloading code and piping it directly to a shell
- Sending likely credentials or secret files through network commands
- Modifying credentials, CI workflows, infrastructure definitions, migrations, or other sensitive paths
- Invoking third-party MCP tools whose effects cannot be classified from their input
- Requesting network, filesystem, or unsandboxed terminal access beyond the sandbox's existing grants

The Auto safety check takes precedence over allow rules, so a high-risk action still asks even when a broad `always_allow` pattern matches.
Built-in catastrophic-operation rules can block an action instead of prompting.
See [Sandboxing](./sandboxing.md) for the default writable locations and network restrictions.

## Quick Start

Use Vela's Settings Editor to [configure tool permissions](vela://settings/agent.tool_permissions), or add the mode and rules directly to your settings file:

```json [settings]
{
  "agent": {
    "permission_mode": "auto",
    "tool_permissions": {
      "tools": {
        "terminal": {
          "always_allow": [
            { "pattern": "^cargo\\s+(build|test|check)" },
            { "pattern": "^npm\\s+(install|test|run)" }
          ],
          "always_confirm": [{ "pattern": "^sudo\\s" }]
        }
      }
    }
  }
}
```

This example uses Auto mode, explicitly approves selected `cargo` and `npm` commands, and requires confirmation for `sudo` commands.
Other unmatched actions use Auto's risk classification.

## How It Works

The permission mode provides the fallback behavior for agent tool actions.
The `tool_permissions` setting adds per-tool defaults and regex patterns that:

- **Auto-approve** actions you trust
- **Auto-deny** dangerous actions
- **Always confirm** sensitive actions

Custom rules apply normally in Manual and Auto modes. In Full Access, explicit deny rules still block actions, while confirmation rules are treated as allowed so the agent does not pause for approval.
These settings apply to Vela's native agent; external agents connected through the Agent Client Protocol (ACP) may also apply their own permission system.

## Supported Tools

| Tool               | Input Matched Against                            |
| ------------------ | ------------------------------------------------ |
| `terminal`         | The shell command string                         |
| `edit_file`        | The file path                                    |
| `write_file`       | The file path                                    |
| `delete_path`      | The path being deleted                           |
| `move_path`        | Source and destination paths                     |
| `copy_path`        | Source and destination paths                     |
| `create_directory` | The directory path                               |
| `fetch`            | The URL                                          |
| `search_web`       | The search query                                 |
| `skill`            | The absolute path to the skill's `SKILL.md` file |

For MCP tools, use the format `mcp:<server>:<tool_name>`.
For example, a tool called `create_issue` on a server called `github` would be `mcp:github:create_issue`.

For model-invoked [Skills](./skills.md), use the `skill` tool. A user-invoked `/skill-name` slash command does not prompt again because you explicitly invoked the skill.

## Configuration

```json [settings]
{
  "agent": {
    "permission_mode": "manual",
    "tool_permissions": {
      "tools": {
        "<tool_name>": {
          "default": "confirm",
          "always_allow": [{ "pattern": "...", "case_sensitive": false }],
          "always_deny": [{ "pattern": "...", "case_sensitive": false }],
          "always_confirm": [{ "pattern": "...", "case_sensitive": false }]
        }
      }
    }
  }
}
```

### Options

| Option                             | Description                                                                   |
| ---------------------------------- | ----------------------------------------------------------------------------- |
| `permission_mode`                  | Overall mode: `"manual"` (default), `"auto"`, or `"full_access"`              |
| `tools.<tool_name>.default`        | Per-tool fallback when no patterns match: `"confirm"`, `"allow"`, or `"deny"` |
| `tools.<tool_name>.always_allow`   | Patterns that auto-approve unless a deny or confirm rule also matches         |
| `tools.<tool_name>.always_deny`    | Patterns that block immediately—highest custom-rule priority                  |
| `tools.<tool_name>.always_confirm` | Patterns that require confirmation unless an `always_deny` rule also matches  |

### Migration from the legacy global default

Existing `agent.tool_permissions.default` values are automatically migrated to `agent.permission_mode` and removed from the settings file:

- `"allow"` becomes `"auto"`
- `"confirm"` becomes `"manual"`
- `"deny"` becomes `"manual"`, because the new model has no global deny-all mode and Manual still prevents unmatched actions from running without approval

An existing explicit `permission_mode` takes precedence. Tool-specific defaults and regex rules remain unchanged.

The removed `agent.default_profile` and `agent.profiles` settings are still accepted when older settings files are loaded, but they no longer select models or change tool availability.

### Pattern Syntax

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "edit_file": {
          "always_allow": [
            {
              "pattern": "your-regex-here",
              "case_sensitive": false
            }
          ]
        }
      }
    }
  }
}
```

Patterns use Rust regex syntax.
Matching is case-insensitive by default.

## Rule Precedence

From highest to lowest priority:

1. **Built-in security rules**: Hardcoded protections (for example, `rm -rf /`) block catastrophic actions.
2. **Invalid custom rules**: A tool with an invalid regex is blocked until the rule is fixed.
3. **`always_deny`**: Blocks matching actions.
4. **`always_confirm`**: Requires confirmation in Manual and Auto.
5. **`always_allow`**: Auto-approves matching actions unless Auto's safety check classifies the action as high risk.
6. **Tool-specific `default`**: Applies when no patterns match (for example, `tools.terminal.default`).
7. **Permission mode fallback**: Uses Confirm for Manual and Allow for Auto or Full Access.

Auto's safety check converts otherwise allowed high-risk actions back to confirmation. Full Access converts confirmation results to allow, but it does not override built-in or explicit deny results.

## Full Access

To skip normal permission confirmations and disable the terminal sandbox:

```json [settings]
{
  "agent": {
    "permission_mode": "full_access"
  }
}
```

Built-in security rules and explicit per-tool deny rules still block matching actions. Confirmation rules do not prompt in Full Access.
Full Access should only be enabled when you understand and accept the risk of unconfirmed, unsandboxed command execution.

## Shell Compatibility

For the `terminal` tool, Vela parses chained commands (e.g., `echo hello && rm file`) to check each sub-command against your patterns.

All supported shells work with tool permission patterns, including sh, bash, zsh, dash, fish, PowerShell 7+, pwsh, cmd, xonsh, csh, tcsh, Nushell, Elvish, and rc (Plan 9).

## Writing Patterns

- Use `\b` for word boundaries: `\brm\b` matches "rm" but not "storm"
- Use `^` and `$` to anchor patterns to start/end of input
- Escape special characters: `\.` for literal dot, `\\` for backslash

<div class="warning">

Test carefully—a typo in a deny pattern blocks legitimate actions.
You can use the "Test Your Rules" checker, available in each individual tool page, to confirm whether a pattern is correctly falling in the desired condition.

</div>

## Built-in Security Rules

Vela includes a small set of hardcoded security rules that **cannot be overridden** by any setting.
These only apply to the **terminal** tool and block recursive deletion of critical directories:

- `rm -rf /` and `rm -rf /*` — filesystem root
- `rm -rf ~` and `rm -rf ~/*` — home directory
- `rm -rf $HOME` / `rm -rf ${HOME}` (and `$HOME/*`) — home directory via environment variable
- `rm -rf .` and `rm -rf ./*` — current directory
- `rm -rf ..` and `rm -rf ../*` — parent directory

These patterns catch any flag combination (e.g., `-fr`, `-rfv`, `-r -f`, `--recursive --force`) and are case-insensitive.
They are checked against both the raw command and each parsed sub-command in chained commands (e.g., `ls && rm -rf /`).

There are no other built-in rules.
The default settings file ({#action vela::OpenDefaultSettings}) includes commented-out examples for protecting `.env` files, secrets directories, and private keys — you can uncomment or adapt these to suit your needs.

## Permission Request in the UI

When the agent requests permission, you'll see in the thread view a tool card with a menu that includes:

- **Allow once** / **Deny once** — One-time decision
- **Always for <tool>** — Sets a tool-level default to allow or deny
- **Always for <pattern>** — Adds an `always_allow` or `always_deny` pattern (when a safe pattern can be extracted)

Selecting "Always for <tool>" sets `tools.<tool>.default` to allow or deny.
When a pattern can be safely extracted, selecting "Always for <pattern>" adds an `always_allow` or `always_deny` rule for that input.
MCP tools only support the tool-level option.

## Examples

### Terminal: Auto-Approve Build Commands

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "terminal": {
          "default": "confirm",
          "always_allow": [
            { "pattern": "^cargo\\s+(build|test|check|clippy|fmt)" },
            { "pattern": "^npm\\s+(install|test|run|build)" },
            { "pattern": "^git\\s+(status|log|diff|branch)" },
            { "pattern": "^ls\\b" },
            { "pattern": "^cat\\s" }
          ],
          "always_deny": [
            { "pattern": "rm\\s+-rf\\s+(/|~)" },
            { "pattern": "sudo\\s+rm" }
          ],
          "always_confirm": [
            { "pattern": "sudo\\s" },
            { "pattern": "git\\s+push" }
          ]
        }
      }
    }
  }
}
```

### File Editing: Protect Sensitive Files

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "edit_file": {
          "default": "confirm",
          "always_allow": [
            { "pattern": "\\.(md|txt|json)$" },
            { "pattern": "^src/" }
          ],
          "always_deny": [
            { "pattern": "\\.env" },
            { "pattern": "secrets?/" },
            { "pattern": "\\.(pem|key)$" }
          ]
        }
      }
    }
  }
}
```

### Path Deletion: Block Critical Directories

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "delete_path": {
          "default": "confirm",
          "always_deny": [
            { "pattern": "^/etc" },
            { "pattern": "^/usr" },
            { "pattern": "\\.git/?$" },
            { "pattern": "node_modules/?$" }
          ]
        }
      }
    }
  }
}
```

### URL Fetching: Control External Access

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "fetch": {
          "default": "confirm",
          "always_allow": [
            { "pattern": "docs\\.rs" },
            { "pattern": "github\\.com" }
          ],
          "always_deny": [{ "pattern": "internal\\.company\\.com" }]
        }
      }
    }
  }
}
```

### MCP Tools

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "mcp:github:create_issue": {
          "default": "confirm"
        },
        "mcp:github:create_pull_request": {
          "default": "confirm"
        }
      }
    }
  }
}
```

### Skills

Patterns for the `skill` tool match against the absolute path to the skill's `SKILL.md` file, not the skill name.

```json [settings]
{
  "agent": {
    "tool_permissions": {
      "tools": {
        "skill": {
          "default": "confirm",
          "always_allow": [{ "pattern": "/code-review/SKILL\\.md$" }]
        }
      }
    }
  }
}
```

To prevent the model from invoking a skill at all, set `disable-model-invocation: true` in that skill's `SKILL.md`. See [Skills](./skills.md#disable-model-invocation).
