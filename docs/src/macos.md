---
title: Vela on macOS
description: "Vela is developed primarily on macOS, making it a first-class platform with full feature support."
---

# Vela on macOS

Vela is developed primarily on macOS, making it a first-class platform with full feature support.

## Installing Vela

Download Vela from the [download page](https://vela.dev/download). The download is a `.dmg` file—open it and drag Vela to your Applications folder.

For the preview build, which receives updates about a week ahead of stable, visit the [preview releases page](https://vela.dev/releases/preview).

After installation, Vela checks for updates automatically and prompts you when a new version is available.

### Homebrew

You can also install Vela using Homebrew:

```sh
brew install --cask vela
```

For the preview version:

```sh
brew install --cask vela@preview
```

### Building from Source

To build Vela from source, see the [macOS development documentation](./development/macos.md).

## System Requirements

- macOS 10.15.7 (Catalina) or later
- Apple Silicon (M1/M2/M3/M4) or Intel processor

Vela uses Metal for GPU-accelerated rendering, which is available on all supported macOS versions.

## Installing the CLI

Vela includes a command-line tool for opening files and projects from Terminal. To install it:

1. Open Vela
2. Open the command palette with `Cmd+Shift+P`
3. Run {#action cli::InstallCliBinary}

This creates a `vela` command in `/usr/local/bin`. You can then open files and folders:

```sh
vela .                    # Open current folder
vela file.txt             # Open a file
vela project/ file.txt    # Open a folder and a file
```

See the [CLI Reference](./reference/cli.md) for all available options.

## Uninstall

1. Quit Vela if it's running
2. Drag Vela from Applications to the Trash
3. Optionally, remove your settings and extensions:

```sh
rm -rf ~/.config/vela
rm -rf ~/Library/Application\ Support/Vela
rm -rf ~/Library/Caches/Vela
rm -rf ~/Library/Logs/Vela
rm -rf ~/Library/Saved\ Application\ State/dev.vela.Vela.savedState
```

If you installed the CLI, remove it with:

```sh
rm /usr/local/bin/vela
```

## Troubleshooting

### Vela won't open or shows "damaged" warning

If macOS reports that Vela is damaged or can't be opened, it's likely a Gatekeeper issue. Try:

1. Right-click (or Control-click) on Vela in Applications
2. Select "Open" from the context menu
3. Click "Open" in the dialog that appears

This tells macOS to trust the application.

If that doesn't work, remove the quarantine attribute:

```sh
xattr -cr /Applications/Vela.app
```

### CLI command not found

If the `vela` command isn't available after installation:

1. Check that `/usr/local/bin` is in your PATH
2. Try reinstalling the CLI via {#action cli::InstallCliBinary} in the command palette
3. Open a new terminal window to reload your PATH

### Can't install CLI {#cant-install-cli}

{#action cli::InstallCliBinary} writes a `vela` symlink to `/usr/local/bin`, which requires administrator privileges. If your macOS account isn't in the `admin` group, Vela can't create that symlink and will report that it can't install the CLI automatically.

Instead, you can add an alias pointing to the `cli` binary bundled inside the app. The path depends on where Vela is installed:

```sh
# Default install (Vela in /Applications)
alias vela="/Applications/Vela.app/Contents/MacOS/cli"

# User install (Vela in ~/Applications)
alias vela="$HOME/Applications/Vela.app/Contents/MacOS/cli"

# Preview build (Vela Preview in ~/Applications)
alias vela="$HOME/Applications/Vela Preview.app/Contents/MacOS/cli"
```

Add the line that matches your install to your shell configuration file. Use `~/.zshrc` for Zsh (the default on modern macOS) or `~/.bashrc` for Bash.

After you restart your shell, you will be able to use `vela` from your terminal:

```sh
vela .              # Open current folder
vela file.txt       # Open a file
```

### GPU or rendering issues

Vela uses Metal for rendering. If you experience graphical glitches:

1. Ensure macOS is up to date
2. Restart your Mac to reset the GPU state
3. Check Activity Monitor for GPU pressure from other apps

### High memory or CPU usage

If Vela uses more resources than expected:

1. Check for runaway language servers in the terminal output ({#action vela::OpenLog})
2. Try disabling extensions one by one to identify conflicts
3. For large projects, consider using [project settings](./reference/all-settings.md#file-scan-exclusions) to exclude unnecessary folders from indexing

For additional help, see the [Troubleshooting guide](./troubleshooting.md) or visit the [Vela Discord](https://discord.gg/vela-community).
