---
title: Uninstall
description: "This guide covers how to uninstall Vela on different operating systems."
---

# Uninstall

This guide covers how to uninstall Vela on different operating systems.

## macOS

### Standard Installation

If you installed Vela by downloading it from the website:

1. Quit Vela if it's running
2. Open Finder and go to your Applications folder
3. Drag Vela to the Trash (or right-click and select "Move to Trash")
4. Empty the Trash

### Homebrew Installation

If you installed Vela using Homebrew, use the following command:

```sh
brew uninstall --cask vela
```

Or for the preview version:

```sh
brew uninstall --cask vela@preview
```

### Removing User Data (Optional)

To completely remove all Vela configuration files and data:

1. Open Finder
2. Press `Cmd + Shift + G` to open "Go to Folder"
3. Delete the following directories if they exist:
   - `~/Library/Application Support/Vela`
   - `~/Library/Saved Application State/dev.vela.Vela.savedState`
   - `~/Library/Logs/Vela`
   - `~/Library/Caches/dev.vela.Vela`
   - `~/Library/Caches/Vela`
   - `~/.config/vela`
   - `~/.local/state/Vela`

## Linux

### Standard Uninstall

If Vela was installed using the default installation script, run:

```sh
vela --uninstall
```

You'll be prompted whether to keep or delete your preferences. After making a choice, you should see a message that Vela was successfully uninstalled.

If the `vela` command is not found in your PATH, try:

```sh
$HOME/.local/bin/vela --uninstall
```

or:

```sh
$HOME/.local/vela.app/bin/vela --uninstall
```

### Package Manager

If you installed Vela using a package manager (such as Flatpak, Snap, or a distribution-specific package manager), consult that package manager's documentation for uninstallation instructions.

### Manual Removal

If the uninstall command fails or Vela was installed to a custom location, you can manually remove:

- Installation directory: `~/.local/vela.app` (or your custom installation path)
- Binary symlink: `~/.local/bin/vela`
- Configuration and data: `~/.config/vela`

## Windows

### Standard Installation

1. Quit Vela if it's running
2. Open Settings (Windows key + I)
3. Go to "Apps" > "Installed apps" (or "Apps & features" on Windows 10)
4. Search for "Vela"
5. Click the three dots menu next to Vela and select "Uninstall"
6. Follow the prompts to complete the uninstallation

Alternatively, you can:

1. Open the Start menu
2. Right-click on Vela
3. Select "Uninstall"

### Removing User Data (Optional)

To completely remove all Vela configuration files and data:

1. Press `Windows key + R` to open Run
2. Type `%APPDATA%` and press Enter
3. Delete the `Vela` folder if it exists
4. Press `Windows key + R` again, type `%LOCALAPPDATA%` and press Enter
5. Delete the `Vela` folder if it exists

## Troubleshooting

If you encounter issues during uninstallation:

- **macOS/Windows**: Ensure Vela is completely quit before attempting to uninstall. Check Activity Manager (macOS) or Task Manager (Windows) for any running Vela processes.
- **Linux**: If the uninstall script fails, check the error message and consider manual removal of the directories listed above.
- **All platforms**: If you want to start fresh while keeping Vela installed, you can delete the configuration directories instead of uninstalling the application entirely.

For additional help, see our [Linux-specific documentation](./linux.md) or visit the [Vela community](https://vela.dev/community-links).
