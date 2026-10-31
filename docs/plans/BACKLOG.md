# Backlog

Ideas and features explicitly postponed. Each entry needs a grill-me session
before it moves into a plan.

| # | Item | Origin | Notes |
|---|------|--------|-------|
| BL1 | Sync custom events between users via the backend (e.g. guild raid schedules) | trailpack-event E6 | Needs accounts/permissions in the backend |
| BL2 | Price trend features (charts, Halloween comparisons) | backend B1, data-pipeline D4 | Needs the backend server; history is archived daily from the start of the data pipeline |
| BL5 | Rust backend server (REST API, Postgres, price history queries) | data-pipeline D1 | Design kept in `backend.md`; seeded from the price archive |
| BL4 | Server backups: Hetzner server backup (+20 %, ~1–1.40 €/month) or nightly `pg_dump` to a Hetzner Storage Box BX11 (~3.80 €/month, 7 daily + 4 weekly, restore test script) | backend B21 | Do it once the price history is valuable |
| BL3 | Private Nextcloud — **not part of trailpack**, kept out of this repo; separate hosting (e.g. Hetzner Storage Share or Storage Box) | grilling round 8 | Never on the public API server (isolation of private data) |
| BL6 | Pin a note as a slim in-game overlay (e.g. daily checklist with clickable checkboxes) | trailpack-memo round 12 | Not in v1 |
