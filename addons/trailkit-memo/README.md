# trailkit-memo

Notes addon for Guild Wars 2, loaded by [Raidcore Nexus](https://raidcore.gg/Nexus).
Plan: [`docs/plans/addons/trailkit-memo.md`](../../docs/plans/addons/trailkit-memo.md).

> **Status:** phase 0 skeleton — a "Hello" window toggled by a keybind
> (unbound by default). Used to validate build → deploy → hot reload.

## Build

From WSL (toolchain is pinned by `rust-toolchain.toml`):

```sh
cargo xtask deploy trailkit-memo            # debug build + copy into the GW2 addons folder
cargo xtask deploy trailkit-memo --release  # release build + copy
```

Build only: `cargo xwin build -p trailkit-memo --target x86_64-pc-windows-msvc`.

## Test

```sh
cargo test -p trailkit-memo   # platform-independent logic (i18n, …) on the host
```

## Layout

| Path | Content |
|------|---------|
| `src/lib.rs` | crate root |
| `src/i18n.rs` | languages + typed UI strings (en/de) |
| `src/addon.rs` | Nexus export, load/unload, render (Windows only) |
| `help/{en,de}.md` | in-game help page source (AGENTS.md rule 6) |
| `package.json` | version placeholder for Changesets (R16) |

## Usage

Set the keybind *"Open/close notes"* under Nexus → Options → Keybinds.
