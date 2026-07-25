---
title: CLI Reference
description: "Reference for Vela's command-line interface (CLI), including opening files and directories, integrating with tools, and controlling Vela from scripts."
---

# CLI Reference

Use Vela's command-line interface (CLI) to open files and directories, integrate with other tools, and control Vela from scripts.

## Installation

**macOS:** Run the {#action cli::InstallCliBinary} command from the command palette ({#kb command_palette::Toggle}) to install the `vela` CLI to `/usr/local/bin/vela`.

**Linux:** The CLI is included with Vela packages. The binary name may vary by distribution (commonly `vela` or `velaitor`).

**Windows:** The CLI is included with Vela. Add Vela's installation directory to your PATH, or use the full path to `vela.exe`.

## Usage

```sh
vela [OPTIONS] [PATHS]...
```

## Opening Files and Directories

Open a file:

```sh
vela myfile.txt
```

Open a directory as a workspace:

```sh
vela ~/projects/myproject
```

Open multiple files or directories:

```sh
vela file1.txt file2.txt ~/projects/myproject
```

Open a file at a specific line and column:

```sh
vela myfile.txt:42        # Open at line 42
vela myfile.txt:42:10     # Open at line 42, column 10
```

## Options

### `-w`, `--wait`

Wait for all opened files to be closed before the CLI exits. When opening a directory, waits until the window is closed.

This is useful for integrating Vela with tools that expect an editor to block until editing is complete (e.g., `git commit`):

```sh
export EDITOR="vela --wait"
git commit  # Opens Vela and waits for you to close the commit message file
```

### `-n`, `--new`

Open paths in a new workspace window, even if the paths are already open in an existing window:

```sh
vela -n ~/projects/myproject
```

### `-a`, `--add`

Add paths to the currently focused workspace instead of opening a new window. When multiple workspace windows are open, files open in the focused window:

```sh
vela -a newfile.txt
```

### `-r`, `--reuse`

Reuse an existing window, replacing its current workspace with the new paths:

```sh
vela -r ~/projects/different-project
```

By default (without `-n`, `-a`, or `-r`), directories open in the current window's sidebar. You can change this default with the `cli_default_open_behavior` setting. See [Windows & Projects](../windows-and-projects.md) for more details.

### `--diff <OLD_PATH> <NEW_PATH>`

Open a diff view comparing two files. Can be specified multiple times:

```sh
vela --diff file1.txt file2.txt
vela --diff old.rs new.rs --diff old2.rs new2.rs
```

### `--foreground`

Run Vela in the foreground, keeping the terminal attached. Useful for debugging:

```sh
vela --foreground
```

### `--user-data-dir <DIR>`

Use a custom directory for all user data (database, extensions, logs) instead of the default location:

```sh
vela --user-data-dir ~/.vela-custom
```

Default locations:

- **macOS:** `~/Library/Application Support/Vela`
- **Linux:** `$XDG_DATA_HOME/vela` (typically `~/.local/share/vela`)
- **Windows:** `%LOCALAPPDATA%\Vela`

### `-v`, `--version`

Print Vela's version and exit:

```sh
vela --version
```

### `--completions <SHELL>`

Generate shell completions for the `vela` CLI:

#### Bash

Add to `~/.bashrc`:

```bash
eval "$(vela --completions bash)"
```

#### Elvish

Add to `~/.config/elvish/rc.elv`:

```elvish
set edit:completion:arg-completer[vela] = { |@args|
    eval (vela --completions elvish | slurp)
    $edit:completion:arg-completer[vela] $@args
}
```

#### Fish

Add to `~/.config/fish/config.fish`:

```fish
vela --completions fish | source
```

#### Nushell

Add to `~/.config/nushell/config.nu`:

```nu
mkdir ($nu.data-dir | path join "vendor/autoload")
^vela --completions nushell | save --force ($nu.data-dir | path join "vendor/autoload/vela.nu")
```

#### Powershell

Add to `$PROFILE`:

```powershell
(&vela --completions powershell) | Out-String | Invoke-Expression
```

#### Zsh

Add to `~/.zshrc`:

```zsh
eval "$(vela --completions zsh)"
```

### `--uninstall`

Uninstall Vela and remove all related files (macOS and Linux only):

```sh
vela --uninstall
```

### `--vela <PATH>`

Specify a custom path to the Vela application or binary:

```sh
vela --vela /path/to/Vela.app myfile.txt
```

## Reading from Standard Input

Read content from stdin by passing `-` as the path:

```sh
echo "Hello, World!" | vela -
cat myfile.txt | vela -
ps aux | vela -
```

This creates a temporary file with the stdin content and opens it in Vela.

## URL Handling

The CLI can open `vela://`, `file://`, and `ssh://` URLs:

```sh
vela vela://settings
vela file:///Users/whatever/.zshrc
vela ssh://me@example.com/abs/path
vela ssh://me@example.com:/abs/path
vela ssh://me@example.com/~/project
vela ssh://me@example.com:~/project
```

## Using Vela as Your Default Editor

Set Vela as your default editor for Git and other tools:

```sh
export EDITOR="vela --wait"
export VISUAL="vela --wait"
```

Add these lines to your shell configuration file (e.g., `~/.bashrc`, `~/.zshrc`).

## macOS: Switching Release Channels

On macOS, you can launch a specific release channel by passing the channel name as the first argument:

```sh
vela --stable myfile.txt
vela --preview myfile.txt
vela --nightly myfile.txt
```

## WSL Integration (Windows)

On Windows, the CLI supports opening paths from WSL distributions. This is handled automatically when launching Vela from within WSL.

## Exit Codes

| Code | Meaning                           |
| ---- | --------------------------------- |
| `0`  | Success                           |
| `1`  | Error (details printed to stderr) |

When using `--wait`, the exit code reflects whether the files were saved before closing.
