# trailkit-api

The Trailkit REST API server. **Status: skeleton only** — the backend is
deferred until a feature needs dynamic queries such as the price history
(ROADMAP phase 6, backlog BL5). The binary builds and does nothing yet.

Target design and decisions: [`docs/plans/backend.md`](../../docs/plans/backend.md).
Layout decisions: ROADMAP R90.

- Runs on Linux as a Docker image behind Caddy (B10, B19), never inside the
  game; the Windows CI job does not build it.
- Shares domain types with the addons and datagen through
  `crates/trailkit-model` once that crate exists (R75).
- Will hold its `sqlx` migrations in `migrations/` and its `Dockerfile` here;
  server-wide configuration (compose, Caddyfile, deploy script) goes to the
  root `deploy/` folder. Each is created when first needed.
- Version `0.0.0`, not managed by Changesets; versioning is decided with the
  first deploy.

## Build and test

```sh
cargo build -p trailkit-api
cargo test -p trailkit-api
```

## Usage

Nothing to run yet.
