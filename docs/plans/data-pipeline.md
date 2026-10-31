# trailkit data pipeline (static data + price archive)

- **Status:** draft — D1–D26 settled
- **Replaces for now:** the always-on server in [`backend.md`](backend.md) (deferred, see D1)
- **Code path (proposal):** `tools/trailkit-datagen` (Rust CLI, run by a scheduled CI job)

## Goal

Provide the data trailkit-core needs (events, items, achievements, waypoints,
current prices) as **static files** that core downloads via plain HTTP(S) —
**free of charge**, without running our own server. In parallel, archive the
trading post prices **daily** in a lossless format so that a Postgres
database with a full price history can be filled later, once a real backend
exists.

## Requirements (from user, 2026-10-31)

- No Rust backend server for now; data lives "somewhere" as files.
- Hosting must be **free**.
- Core behaves as planned (cache, ETag, daily refresh, offline from disk) —
  only the source changes from REST endpoints to static files.
- A **daily cron job** archives prices to files. Store *everything* needed
  for later price-trend analysis ("rather too much than too little").
- The archive is **not queryable** and not used by any addon yet; it only
  exists to seed a future Postgres database.
- **All** data files (items, achievements, … too) are structured so they can
  be imported into Postgres later. Only prices are archived; the other data
  sets keep only the current state.
- No price history UI in any addon for now (→ backlog BL2).

## Architecture

```
scheduled CI job (daily, UTC)
  └─ trailkit-datagen (Rust CLI)
       ├─ GW2 API ──► items, achievements, waypoints, prices, build id
       ├─ data/events/ (curated, in this repo)
       ├─ writes "current" snapshot  ──► static host  ◄── trailkit-core (HTTP GET + ETag)
       └─ writes price archive file   ──► archive storage (never read by addons)
```

- One Rust CLI does both jobs; it reuses the DTOs in
  `crates/trailkit-api-types` that core uses to parse the files
  (end-to-end typing, same as the planned REST API).
- **Paths mirror the future REST API** (`/v1/items.en.json`,
  `/v1/prices.json`, `/v1/meta.json`, …), so a later server can serve the
  same paths and core only needs a base-URL change (or none, if a domain we
  own points at the host).
- `meta.json` lists every data set with `generated_at`, `gw2_build`,
  `schema_version`, size and hash → core fetches `meta.json` first
  (tiny, ETag) and only downloads data sets whose hash changed.

## Postgres-readiness rules (all data files)

- Every record has a stable primary key (GW2 `id`; composite keys spelled
  out, e.g. `(item_id, captured_at)`).
- Flat, typed fields with fixed names in `snake_case`; nested GW2 structures
  only where Postgres would use `jsonb` anyway (documented per data set).
- Timestamps in UTC, RFC 3339; money as integer copper; no locale-formatted
  values.
- Each data set documents its target table (`CREATE TABLE` sketch) in
  `tools/trailkit-datagen/README.md`; `schema_version` bumps on every
  breaking change.
- Localized fields (`name`, `description`) per language file
  (`items.en.json`, `items.de.json`) → table `item_i18n(item_id, lang, …)`.

## Price archive (proposal — see round 12a)

- Captured once per day: full `/v2/commerce/prices` (all ~28k tradeable ids:
  `id`, `whitelisted`, `buys.{quantity,unit_price}`,
  `sells.{quantity,unit_price}`) **stored raw and lossless** as returned by
  the API, one JSON object per line (NDJSON, gzip), plus a per-run manifest
  (`captured_at` start/end, `gw2_build`, id count, failed ids, tool version).
- Candidates for "rather too much" (decision pending): full order book
  `/v2/commerce/listings` (much larger), gem exchange rates
  `/v2/commerce/exchange/{coins,gems}`.
- Never read by addons; import into Postgres later via a documented script
  (`COPY` into a staging `jsonb` table, then typed tables).

## Hosting options (research 2026-10-31)

| Option | Cost | Fit | Limits / concerns |
|--------|------|-----|-------------------|
| **GitHub Actions** (cron) | free for public repos | runs the daily job | schedule may be delayed under load (start of hour worst); scheduled workflows **auto-disabled after 60 days without repo activity** (the archive makes no commits → self re-enable, D38); US provider, but the job handles no personal data |
| **GitHub Pages** (serve current files) | free | static files, CDN, ETag, gzip | soft limits 1 GB site, 100 GB/month bandwidth; US provider → every addon user's IP reaches GitHub (conflicts with B20) |
| **GitHub Releases** (archive) | free | large assets (≤ 2 GB per file), not in git history | US provider (irrelevant for the archive: no personal data) |
| Codeberg Pages / Actions | free, EU (Berlin) | would fit B20 | **excluded:** Codeberg ToS 2.7 (July 2026) bans projects mostly written by generative AI; also storage quota 750 MiB repo + 1.5 GiB extra |
| Bunny.net Storage + CDN | ~1 €/month minimum | EU, fits B20 | not free |
| Hetzner Object Storage | ~5 €/month | EU, fits B20 | not free |
| Cloudflare R2 / Pages | free | technically ideal | excluded by B20 (US) |

Observation: Nexus addon updates are typically distributed via GitHub
Releases, so users' clients already contact GitHub for trailkit updates.

## Measurements (2026-10-31, 600 random tradeable ids, extrapolated to 27 999)

| Data | Raw / day | gzip -9 / day | gzip / year |
|------|-----------|---------------|-------------|
| `/v2/commerce/prices` (all ids) | 3.3 MB | **0.44 MB** | ~160 MB |
| `/v2/commerce/listings` (full order book) | 228 MB | **19.5 MB** | ~7.1 GB |
| order book, best 20 tiers per side + totals | 43 MB | **4.1 MB** | ~1.5 GB |
| order book, best 5 tiers + depth buckets (D17) | — | **1.9 MB** | ~690 MB |
| `/v2/wvw/matches?ids=all` (one snapshot) | 0.36 MB | **0.03 MB** | ~10 MB daily (+ pre-reset snapshots) |
| `/v2/commerce/exchange/*`, `/v2/build` | < 1 KB | < 1 KB | negligible |

Order book example (Glob of Ectoplasm, id 19721): 331 buy tiers and
3 472 sell tiers — the long tail of far-off sell offers is what makes the
full book large. Tiers per item: median 108, average 174, max 2 166.

Served item file (per language, 74 411 items): B15 fields only
15.9 MB raw / **4.1 MB gz**; full GW2 item objects 45.6 MB raw / 8.2 MB gz.

Storage limits: GitHub Pages only holds the *current* files (soft limit
1 GB site) — the archive is **not** on Pages. GitHub Releases: each file
< 2 GiB, no documented limit on total release size or bandwidth (fair use
still applies).

## Traffic budget (GitHub Pages soft limit 100 GB/month)

- Core only downloads a data set when `meta.json` shows a changed hash (D10)
  **and** an installed addon needs it (core data set registry).
- Items (~4 MB gz per language) change only with game patches → a few
  downloads per month per user.
- Prices (~1 MB gz) change daily → ~30 MB/month per user that needs prices.
  At ~3 000 such users this reaches the 100 GB soft limit → then move to our
  own domain + EU CDN (D11). No current addon needs prices.

## Archive layout

- One GitHub Release per month (`prices-YYYY-MM`), **one file per data set
  per day** (e.g. `2026-11-01.prices.ndjson.gz`,
  `2026-11-01.orderbook.ndjson.gz`, `2026-11-01.wvw.ndjson.gz`,
  `2026-11-01.manifest.json`) → ~6 files/day, ~190 assets per release
  (GitHub limit: 1 000 assets per release, 2 GiB per file).
- Expected size per monthly release: prices ~13 MB + order book ~57 MB +
  WvW ~1 MB ≈ **~72 MB/month, ~860 MB/year**.

## GDPR assessment (not legal advice)

- Every core download sends the user's IP address to GitHub (Microsoft, US).
- Transfer basis: EU-US Data Privacy Framework adequacy decision (2023);
  GitHub/Microsoft are certified. Upheld by the General Court (T-553/23,
  Sept 2025); appeal C-703/25 P pending at the CJEU → a "Schrems III" ruling
  could end it, which is why D11 (own domain, movable base URL) matters.
- Processing basis: legitimate interest (Art. 6(1)(f)) — downloading game
  data is necessary for the addon's function.
- Required: transparent notice (who receives the IP, why, transfer basis) in
  core's help page and the repo/website privacy notice.
- Additional mitigation (D14): core downloads nothing before a one-time
  first-start confirmation.
- The CJEU appeal status is a recurring check (AGENTS.md "Recurring checks")
  plus D18 (alert + monthly reminder issue).

## Open questions (grilling round 12b)

See `NEXT_SESSION.md`.

## Decisions

| # | Decision | Date |
|---|----------|------|
| D1 | **No backend server for now.** Static files + scheduled job instead; the server plan in `backend.md` is deferred until a feature needs dynamic queries (e.g. price history). Supersedes B8 for the current phases | 2026-10-31 |
| D2 | Hosting of the data pipeline must be **free** | 2026-10-31 |
| D3 | trailkit-core fetches the files via HTTP(S) and keeps its planned behavior (disk cache, ETag/304, daily refresh with jitter, offline start) | 2026-10-31 |
| D4 | Prices are archived **daily** in a lossless, Postgres-importable format; the archive is write-only (no addon reads it) | 2026-10-31 |
| D5 | All published data sets follow the Postgres-readiness rules above; only prices are archived, other sets keep the current state only | 2026-10-31 |
| D6 | Current data files are served via **GitHub Pages** — documented exception to B20 for public game data delivery; GDPR notice required (see assessment) | 2026-10-31 |
| D7 | Scheduled job: GitHub Actions in this repo, daily 00:17 UTC + catch-up run 06:17 UTC that only acts if today's archive is missing or incomplete (amended by D35) | 2026-10-31 |
| D8 | One repo: current files on orphan branch `gh-pages` (force-pushed, no history); archive as monthly GitHub Releases `prices-YYYY-MM`, one asset per day | 2026-10-31 |
| D9 | The CI bot may push `gh-pages` and create archive releases without per-push approval; changes to the workflow itself still need approval (AGENTS.md rule 1) | 2026-10-31 |
| D10 | Formats: archive = raw NDJSON.gz + manifest; served files = JSON per data set and language under `/v1/…` + `meta.json` with hashes | 2026-10-31 |
| D11 | Core uses the GitHub Pages URL until the first public release, then our own domain (B13; settled as `gw2.trailkit.de`, ROADMAP R52); `meta.json` can announce a new base URL that old core versions follow | 2026-10-31 |
| D12 | Prices for core: one file with all current prices (~1 MB gz), refreshed **twice daily** (amended by D40) — it is only the starting value; core refreshes live on demand (core C14) | 2026-10-31 |
| D13 | Order: the price archive starts right after the phase 0 toolchain, before trailkit-memo | 2026-10-31 |
| D14 | Core shows a one-time first-start notice ("game data is loaded from GitHub, USA; your IP address is transmitted") and downloads nothing until the user confirms; same notice in core's help page and the privacy policy | 2026-10-31 |
| D15 | Additionally archived: gem exchange rates, `/v2/build`, ~~daily item-id diff~~ (dropped, D30), **WvW matches** (daily + snapshots shortly before the EU/NA weekly resets for final scores; ~10–20 MB/year). Never archived: anything requiring an API key | 2026-10-31 |
| D16 | Archive retention: **keep everything forever** (no deletion); supersedes the 1-year retention of B1 for the archive | 2026-10-31 |
| D17 | Order book archived as **best 5 tiers per side + market depth buckets** (total quantity within +1 %, +5 %, +10 % of the best price, plus total quantity and tier count per side); measured in the first week | 2026-10-31 |
| D18 | Ruling alert for C-703/25 P: user sets a Google Alert ("C-703/25", "Latombe Data Privacy Framework"); a monthly GitHub Actions job opens a reminder issue "check C-703/25 P" (GitHub emails it) | 2026-10-31 |
| D19 | Stay GitHub-only for now (B20 unchanged, Cloudflare not used). `meta.json` and small data sets on GitHub Pages; large files (items, prices) as **GitHub Release assets with the content hash in the file name**, so the Pages 100 GB/month soft limit does not apply. Later move to own domain + EU CDN via `meta.json` without an addon update | 2026-10-31 |
| D20 | Waypoints: **all waypoints in the game** — every continent (Tyria, the Mists) and every floor, deduplicated by point-of-interest id; fields: id, name (en/de), map id, coordinates, chat link | 2026-10-31 |
| D21 | Achievements: full objects + categories + groups are published; core only downloads them when a consumer needs them (C10) | 2026-10-31 |
| D22 | Item file fields: B15 fields (id, name, icon, rarity, type, level, chat link) **+ `vendor_value` + `flags`** | 2026-10-31 |
| D23 | Event schedules are hand-maintained **TOML** files, one per category, in `data/events/` (format in `data/events/README.md`); datagen validates them and generates `/v1/events.json` | 2026-10-31 |
| D24 | `data/events/` has its own license **CC BY-NC-SA 2.5** (`data/events/LICENSE`, derived from the GW2 wiki); attribution included in `events.json`; the rest of the repo stays MIT; event data must never be monetized | 2026-10-31 |
| D25 | Schema evolution of served files: adding fields is always allowed (consumers ignore unknown fields); breaking changes get a new path prefix (`/v2/…`); the old prefix keeps being generated for **6 months** after the new one ships | 2026-10-31 |
| D26 | datagen is a polite GW2 API client: client-side throttle (~4 req/s), retries with backoff on 429/5xx, partial failures recorded in the manifest instead of aborting the whole run | 2026-10-31 |
| D27 | Code path: `tools/trailkit-datagen` (binary crate in the workspace) | 2026-10-31 |
| D28 | GW2 API DTOs live inside datagen for now (only the order book condensation D17 needs typed parsing; the archive stays raw); they move to `crates/trailkit-api-types` when core needs them (AGENTS.md rule 7) | 2026-10-31 |
| D29 | WvW reset snapshots (D15) at **Fri 17:23 UTC** (before EU reset 18:00) and **Sat 01:23 UTC** (before NA reset 02:00): ~35 min margin for delayed scheduled runs, off the full hour; one asset per snapshot (`<date>.wvw-eu-reset.ndjson.gz`, `<date>.wvw-na-reset.ndjson.gz`) | 2026-10-31 |
| D30 | **No item-id diff** (supersedes that part of D15): the daily price archive already contains every tradeable id, so new/removed tradeable items are visible there; non-tradeable items have no price relevance, and the current item list can be rebuilt from the API at any time. Saves code (previous-day lookup) and archive space | 2026-10-31 |
| D31 | Gem exchange (D15) archived at the in-game quantities used as amounts paid: `/v2/commerce/exchange/gems` with 400, 800, 1200, 2000 gems ("Get Gold" direction) and `/v2/commerce/exchange/coins` with 1, 10, 50, 100, 250 gold ("Get Gems" direction) — 9 requests per run | 2026-10-31 |
| D32 | datagen libraries: `clap` (derive) for the CLI, `flate2` (pure-Rust backend, level 9) for gzip, `log` + `env_logger` for output, `ureq` (R65), `jiff` (R66), `anyhow` for errors | 2026-10-31 |
| D33 | Daily files: `<date>.prices.ndjson.gz` (raw API objects, unchanged except insignificant whitespace removed — the API pretty-prints, NDJSON needs one line per object), `<date>.orderbook.ndjson.gz` (D17), `<date>.wvw.ndjson.gz`, `<date>.gem-exchange.json.gz` (D31), `<date>.manifest.json` (start/end, `gw2_build` from `/v2/build`, tool version, per data set: status, `captured_at`, record count, failed ids, error, size, SHA-256). Reset snapshots via `wvw-snapshot --region eu\|na` → `<date>.wvw-<region>-reset.ndjson.gz` | 2026-10-31 |
| D34 | Roles: datagen only writes to a local directory and never talks to GitHub; the workflow creates the monthly release, uploads assets and decides what to (re)run, using the `gh` CLI. datagen needs no token and is side-effect free locally | 2026-10-31 |
| D35 | Failure handling: per request up to 5 attempts with exponential backoff (D26); per data set a status `complete` / `partial` (`failed_ids`) / `failed` (`error`) in the manifest, other data sets continue; the catch-up run (D7, amended) re-runs every data set of the day that is missing, `failed` or `partial` and replaces its asset + manifest; if anything is still incomplete after the catch-up the workflow fails → GitHub notification email. Missed days cannot be backfilled (the API has no history) and stay documented as gaps | 2026-10-31 |
| D36 | Tests use small committed API response fixtures; CI never calls the live GW2 API; a manual local run validates against the live API before the workflow is enabled | 2026-10-31 |
| D37 | `webpki-roots` (Mozilla CA list, pulled in by `ureq`/`rustls`) is licensed CDLA-Permissive-2.0: allowed via a `cargo-deny` exception for that crate only, not as a general license allowance | 2026-10-31 |
| D38 | Keep-alive: GitHub disables scheduled workflows after 60 days without repository activity and the archive produces no commits, so every archive run re-enables its own workflow via the API (`gh workflow enable`, `actions: write`) | 2026-10-31 |
| D39 | Monthly release `prices-YYYY-MM`: tag on the commit the scheduled run uses — the default branch `develop` since R67 — created with `--latest=false` so "Latest release" stays an addon release, body explains the content and links the datagen README | 2026-10-31 |
| D40 | The served prices file (D12, phase 2) is generated **twice daily**: 00:17 UTC from the archive run's price fetch (no extra API requests, right after the daily reset, NA prime time) and 16:17 UTC with a prices-only fetch (before EU prime time). Old files are deleted automatically (keep the last 4). The archive itself stays one snapshot per day and one release per month | 2026-10-31 |
