# Backlog

Ideas and features explicitly postponed. Each entry needs a grill-me session
before it moves into a plan.

| # | Item | Origin | Notes |
|---|------|--------|-------|
| BL1 | Sync custom events between users via the backend (e.g. guild raid schedules) | trailkit-event E6 | Needs accounts/permissions in the backend |
| BL2 | Price trend features (charts, Halloween comparisons) | backend B1, data-pipeline D4 | Needs the backend server; history is archived daily from the start of the data pipeline |
| BL5 | Rust backend server (REST API, Postgres, price history queries) | data-pipeline D1 | Design kept in `backend.md`; seeded from the price archive |
| BL4 | Server backups: Hetzner server backup (+20 %, ~1–1.40 €/month) or nightly `pg_dump` to a Hetzner Storage Box BX11 (~3.80 €/month, 7 daily + 4 weekly, restore test script) | backend B21 | Do it once the price history is valuable |
| BL3 | Private Nextcloud — **not part of trailkit**, kept out of this repo; separate hosting (e.g. Hetzner Storage Share or Storage Box) | grilling round 8 | Never on the public API server (isolation of private data) |
| BL6 | Pin a note as a slim in-game overlay (e.g. daily checklist with clickable checkboxes) | trailkit-memo round 12 | Not in v1 |
| BL8 | Note editor with line wrapping (ImGui 1.80's multiline input cannot wrap; own editor needs cursor, selection, clipboard, undo, IME) | trailkit-memo M53 | Until then long lines scroll sideways in edit mode |
| BL7 | Sell calculator: for an item and quantity show "sell instantly" (walk down the buy orders, fees per transaction) vs. "best case" (list at lowest sell offer − 1 copper, fees), with the game's fee rounding (listing fee 5 %, exchange fee 10 %, each rounded to whole copper, min 1) | discussion 2026-10-31 | Needs core prices (C14); live order book per item via `/v2/commerce/listings?ids=<item>` |
