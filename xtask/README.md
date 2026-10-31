# xtask

Repository task runner ([cargo-xtask pattern](https://github.com/matklad/cargo-xtask)),
invoked from the repo root as `cargo xtask <command>` (alias in `.cargo/config.toml`).

## Commands

| Command | What it does |
|---------|--------------|
| `cargo xtask deploy <addon> [--release]` | Builds `addons/<addon>` with `cargo xwin` for `x86_64-pc-windows-msvc` and copies the DLL into the GW2 `addons` folder (copy to `*.dll.tmp`, then rename) so Nexus hot-reloads it. |
| `cargo xtask check-branch [--head B] [--base B]` | Verifies a branch name (R14); without options the current branch. Used by CI and the pre-push hook. |
| `cargo xtask install-hooks` | Enables the versioned git hooks in `.githooks/` (pre-commit: fmt + names; pre-push: branch + clippy + tests). |
| `cargo xtask check-names` | Verifies every file/directory name against [AGENTS.md → Naming conventions](../AGENTS.md#naming-conventions). |
| `cargo xtask sync-versions` | Copies each addon's `package.json` version (owned by Changesets) into its `Cargo.toml`. |
| `cargo xtask sync-versions --check` | Only verifies the versions match (CI). |

## Configuring the GW2 addons folder

Never hard-coded. Either set an environment variable:

```sh
export TRAILKIT_GW2_ADDONS_DIR="/mnt/c/Guild Wars 2/addons"
```

or create the git-ignored `xtask.local.toml` at the repo root:

```toml
gw2_addons_dir = "/mnt/c/Guild Wars 2/addons"
```

The environment variable wins over the file.

## Requirements

- `cargo install --locked cargo-xwin`
- First build downloads the MSVC CRT + Windows SDK; this requires accepting the
  Microsoft license (`XWIN_ACCEPT_LICENSE=1`, ROADMAP R30).
