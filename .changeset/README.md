# Changesets

Every work branch adds a changeset describing its user-visible change
(ROADMAP R16):

```sh
pnpm changeset
```

Merges to `develop` create no versions. On `main`, the Changesets action opens
the version PR; `pnpm version` bumps `package.json` versions, writes each
addon's `CHANGELOG.md` and syncs the versions into `Cargo.toml`
(`cargo xtask sync-versions`). Releases are tagged `<addon>@x.y.z`.

Docs: <https://github.com/changesets/changesets>
