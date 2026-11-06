# Changesets

Every work branch normally adds a changeset describing its user-visible
change (ROADMAP R16). **Until memo's first release**, its work branches do
not need individual changesets: release preparation adds one consolidated
memo changeset covering the unreleased changes (Q142, 2026-11-06). This
exception ends with the first release; keep decisions and behavior documented
in the plans and help pages throughout development.

To create a changeset:

```sh
pnpm changeset
```

Merges to `develop` create no versions. On `main`, the Changesets action opens
the version PR; `pnpm version` bumps `package.json` versions, writes each
addon's `CHANGELOG.md` and syncs the versions into `Cargo.toml`
(`cargo xtask sync-versions`). Releases are tagged `<addon>@x.y.z`.

Docs: <https://github.com/changesets/changesets>
