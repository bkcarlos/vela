# Development environment status

Checked on 2026-07-23.

| Requirement | Status | Detail |
|---|---|---|
| Host | Ready | macOS 26.5.2, arm64 |
| Vela checkout | Ready | Clean at `6297c88f428a99741a7bfb33f31dfe98123bb8e4` |
| Pi reference checkout | Ready | Clean at `9b3a2059171bcc74ad9d2cadeea6d186776cf2db` |
| Rust 1.95.0 | Ready | Installed with Vela-required components and targets |
| Xcode | Ready | Xcode 26.5 at `/Applications/Xcode.app`; wrapper exports `DEVELOPER_DIR` |
| Metal Toolchain | Ready | Metal Toolchain 17F42 installed through Xcode |
| cmake | Ready | Homebrew cmake 4.4.0 |
| Vela compile check | Ready | `./scripts/vela-cargo.sh check -p vela` passes |
| Vela debug build | Ready | `./scripts/vela-cargo.sh build -p vela` passes; isolated smoke launch stays healthy |
| Node.js | Ready | v22.19.0; reference tooling only |

## Usage

The global `xcode-select` still points at Command Line Tools because changing it requires administrator authentication. Run Vela commands through the wrapper, which selects the installed Xcode without changing global state:

```bash
./scripts/vela-cargo.sh check -p vela
```

Re-run `scripts/check-dev-environment.sh` after toolchain or Xcode changes.
