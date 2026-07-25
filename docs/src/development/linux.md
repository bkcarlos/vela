---
title: Building Vela for Linux
description: "Guide to building vela for linux for Vela development."
---

# Building Vela for Linux

## Repository

Clone the [Vela repository](https://github.com/vela-industries/vela).

## Dependencies

- Install [rustup](https://www.rust-lang.org/tools/install)

- Install the necessary system libraries:

  ```sh
  script/linux
  ```

  If you prefer to install the system libraries manually, you can find the list of required packages in the `script/linux` file.

## Building from source

Once the dependencies are installed, you can build Vela using [Cargo](https://doc.rust-lang.org/cargo/).

For a debug build of the editor:

```sh
cargo run
```

And to run the tests:

```sh
cargo test --workspace
```

In release mode, the primary user interface is the `cli` crate. You can run it in development with:

```sh
cargo run -p cli
```

## Installing a development build

You can install a local build on your machine with:

```sh
./script/install-linux
```

This builds `vela` and the `cli` in release mode, installs the binary at `~/.local/bin/vela`, and installs `.desktop` files to `~/.local/share`.

## Wayland & X11

Vela supports both X11 and Wayland. By default, we pick whichever we can find at runtime. If you're on Wayland and want to run in X11 mode, use the environment variable `WAYLAND_DISPLAY=''`.

## Notes for packaging Vela

This section is for distribution maintainers packaging Vela.

### Technical requirements

Vela has two main binaries:

- You will need to build `crates/cli` and make its binary available in `$PATH` with the name `vela`.
- You will need to build `crates/vela` and put it at `$PATH/to/cli/../../libexec/vela-editor`. For example, if you are going to put the cli at `~/.local/bin/vela` put vela at `~/.local/libexec/vela-editor`. As some linux distributions (notably Arch) discourage the use of `libexec`, you can also put this binary at `$PATH/to/cli/../../lib/vela/vela-editor` (e.g. `~/.local/lib/vela/vela-editor`) instead.
- If you are going to provide a `.desktop` file you can find a template in `crates/vela/resources/vela.desktop.in`, and use `envsubst` to populate it with the values required. This file should also be renamed to `$APP_ID.desktop` so that the file [follows the FreeDesktop standards](https://github.com/vela-industries/vela/issues/12707#issuecomment-2168742761). You should also make this desktop file executable (`chmod 755`).
- You will need to ensure that the necessary libraries are installed. You can get the current list by [inspecting the built binary](https://github.com/vela-industries/vela/blob/935cf542aebf55122ce6ed1c91d0fe8711970c82/script/bundle-linux#L65-L67) on your system.
- For an example of a complete build script, see [script/bundle-linux](https://github.com/vela-industries/vela/blob/935cf542aebf55122ce6ed1c91d0fe8711970c82/script/bundle-linux).
- You can disable Vela's auto updates and provide instructions for users who try to update Vela manually by building (or running) Vela with the environment variable `VELA_UPDATE_EXPLANATION`. For example: `VELA_UPDATE_EXPLANATION="Please use flatpak to update vela."`.
- Make sure to update the contents of the `crates/vela/RELEASE_CHANNEL` file to 'nightly', 'preview', or 'stable', with no newline. This will cause Vela to use the credentials manager to remember a user's login.

### Other things to note

Vela moves quickly, and distribution maintainers often have different constraints and priorities. The points below describe current trade-offs:

- Vela is a fast-moving project. We typically publish 2-3 builds per week to address reported issues and ship larger changes.
- There are a couple of other `vela` binaries that may be present on Linux systems ([1](https://openzfs.github.io/openzfs-docs/man/v2.2/8/vela.8.html), [2](https://vela.brimdata.io/docs/commands/vela)). If you want to rename our CLI binary because of these issues, we suggest `velait`, `velaitor`, or `vela-cli`.
- Vela automatically installs versions of common developer tools, similar to rustup/rbenv/pyenv. This behavior is discussed [here](https://github.com/vela-industries/vela/issues/12589).
- Users can install extensions locally and from [vela-industries/extensions](https://github.com/vela-industries/extensions). Extensions may install additional tools such as language servers. Planned safety improvements are tracked [here](https://github.com/vela-industries/vela/issues/12358).
- Vela connects to several online services by default (AI, telemetry, collaboration). AI and our telemetry can be disabled by your users with their vela settings or by patching our [default settings file](https://github.com/vela-industries/vela/blob/main/assets/settings/default.json).
- Because of the points above, Vela currently does not work well with sandboxes. See [this discussion](https://github.com/vela-industries/vela/pull/12006#issuecomment-2130421220).

## Flatpak

> Vela's current Flatpak integration exits the sandbox on startup. Workflows that rely on Flatpak's sandboxing may not work as expected.

To build & install the Flatpak package locally follow the steps below:

1. Install Flatpak for your distribution as outlined [here](https://flathub.org/setup).
2. Run the `script/flatpak/deps` script to install the required dependencies.
3. Run `script/flatpak/bundle-flatpak`.
4. Now the package has been installed and has a bundle available at `target/release/{app-id}.flatpak`.

## Memory profiling

[`heaptrack`](https://github.com/KDE/heaptrack) is quite useful for diagnosing memory leaks. To install it:

```sh
$ sudo apt install heaptrack heaptrack-gui
$ cargo install cargo-heaptrack
```

Then, to build and run Vela with the profiler attached:

```sh
$ cargo heaptrack -b vela
```

When this vela instance is exited, terminal output will include a command to run `heaptrack_interpret` to convert the `*.raw.zst` profile to a `*.zst` file which can be passed to `heaptrack_gui` for viewing.

## Perf recording

How to get a flamegraph with resolved symbols from a running Vela instance.
Use this when Vela is using a lot of CPU. It is not useful for hangs.

### During the incident

- Find the PID (process ID) using:
  `ps -eo size,pid,comm | grep vela | sort | head -n 1 | cut -d ' ' -f 2`
  Or find the PID of `vela-editor` with the highest RAM usage in something
  like htop/btop/top.

- Install perf:
  On Ubuntu (derivatives) run `sudo apt install linux-tools`.

- Perf record:
  Run `sudo perf record -p <pid you just found>`, wait a few seconds to gather data, then press Ctrl+C. You should now have a `perf.data` file.

- Make the output file user owned:
  run `sudo chown $USER:$USER perf.data`

- Get build info:
  Run vela again and type {#action vela::About} in the command pallet to get the exact commit.

The `perf.data` file can be sent to Vela together with the exact commit.

### Later

This can be done by Vela staff.

- Build Vela with symbols:
  Check out the commit found previously and modify `Cargo.toml`.
  Apply the following diff, then make a release build.

```diff
[profile.release]
-debug = "limited"
+debug = "full"
```

- Add the symbols to the perf database:
  `perf buildid-cache -v -a <path to release vela binary>`

- Resolve the symbols from the db:
  `perf inject -i perf.data -o perf_with_symbols.data`

- Install flamegraph:
  `cargo install cargo-flamegraph`

- Render the flamegraph:
  `flamegraph --perfdata perf_with_symbols.data`

## Troubleshooting

### Cargo errors claiming that a dependency is using unstable features

Try `cargo clean` and `cargo build`.
