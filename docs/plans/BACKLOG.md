# Backlog

Ideas and features explicitly postponed. Each entry needs a grill-me session
before it moves into a plan.

| # | Item | Origin | Notes |
|---|------|--------|-------|
| BL1 | Sync custom events between users via the backend (e.g. guild raid schedules) | trailkit-event E6 | Needs accounts/permissions in the backend |
| BL2 | Price trend features (charts, Halloween comparisons) | backend B1, data-pipeline D4 | Needs the backend server; history is archived daily from the start of the data pipeline |
| BL5 | Rust backend server (REST API, Postgres, price history queries) | data-pipeline D1 | Design kept in `backend.md`; seeded from the price archive |
| BL4 | Server backups: Hetzner server backup (+20 %, ~1–1.40 €/month) or nightly `pg_dump` to a Hetzner Storage Box BX11 (~3.80 €/month, 7 daily + 4 weekly, restore test script) | backend B21 | Do it once the price history is valuable |
| BL10 | "Open in wiki" action on chat-link chips | trailkit-memo M75 | The GW2 wiki is behind a login since 2026-10-31 (see `data/events/README.md`); revisit when it is public again |
| BL3 | Private Nextcloud — **not part of trailkit**, kept out of this repo; separate hosting (e.g. Hetzner Storage Share or Storage Box) | grilling round 8 | Never on the public API server (isolation of private data) |
| BL6 | Pin a note as a slim in-game overlay (e.g. daily checklist with clickable checkboxes) | trailkit-memo round 12 | Not in v1 |
| BL7 | Sell calculator: for an item and quantity show "sell instantly" (walk down the buy orders, fees per transaction) vs. "best case" (list at lowest sell offer − 1 copper, fees), with the game's fee rounding (listing fee 5 %, exchange fee 10 %, each rounded to whole copper, min 1) | discussion 2026-10-31 | Needs core prices (C14); live order book per item via `/v2/commerce/listings?ids=<item>` |
| BL9 | Flipping helper: show how often an item actually sells (sold/bought per day), not only prices | discussion 2026-11-01 | Source **DataWars2** (`api.datawars2.ie`, operator Silver, Ireland/EU): on 2026-11-01 Silver allowed (1) using the API from the addon without limits beyond reasonable use, (2) archiving its data in our public daily archive, (3) attribution wording/link up to us; (4) no deliberate IP logging, nginx may keep IPs until its log rotates → name DataWars2 in the privacy notice (GDPR). Caveats from our own check: estimates differ from GW2BLTC by factor 3–20 (DataWars2 `listed` likely overstated by a branch in `calculate_sold`; cancellations at the best price can count as sold); one hour of own 1-minute polling matched Mithril roughly but not ecto/mystic coin → show volumes as multi-day medians/ranks, not exact counts. Optional later: own collector in the backend (BL5); archiving DataWars2 daily snapshots (`/gw2/v1/items/json?fields=id,1d_sell_sold,…`) needs a grill round + workflow approval. GW2BLTC asked by e-mail (draft given to the user), no answer yet |
