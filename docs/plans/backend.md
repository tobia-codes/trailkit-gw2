# trailpack backend

- **Status:** **deferred** (2026-10-31, see [`data-pipeline.md`](data-pipeline.md) D1). Kept as the
  target design for when a dynamic feature (e.g. price history) needs a server.
  Data formats and paths of the static files are chosen so this server can
  take over without changing core.
- **Crate path:** `backend/` (binary) + shared types in `crates/trailpack-api-types`

## Goal

A Rust service that mirrors the data the trailpack addons need from the
official Guild Wars 2 API into our own database, so addons never hit the
GW2 API for static/shared data and keep working when the GW2 API is down.
It also archives trading post prices to enable price history features.

## Requirements (from user)

- Data sets: **events** (schedules), **waypoints**, **items**,
  **achievements**, **item prices** (trading post).
- Only **prices** are archived historically. Items, events, waypoints,
  achievements keep only the current state.
- Scheduled refresh (cron) with different cadences for fast-changing data
  (prices) and slow-changing data (items, achievements, waypoints, events).
- Free hosting (Neon was suggested; alternatives welcome).
- Security: rate limiting; ideally only trailpack addons can query the API.
- Player-specific data (API key) is **not** handled by the backend — that is
  core-addon ↔ GW2 API directly.

## Research findings (2026-10-31)

| Data | GW2 API source | Size | Notes |
|------|----------------|------|-------|
| Items | `/v2/items` | 74 411 ids | Up to 200 ids per request → ~373 requests for full sync. Changes on game patches. Localized (`lang=en/de`). |
| Prices | `/v2/commerce/prices` | 27 999 tradeable ids | 200 per request → ~140 requests per full snapshot. |
| Achievements | `/v2/achievements` (+ `/categories`, `/groups`) | 8 357 | Changes on patches. Localized. |
| Waypoints | `/v2/continents/1/floors/{n}` → `regions.maps.points_of_interest` (`type = waypoint`, `chat_link`) | floor 1 ≈ 7.9 MB JSON | Changes rarely. |
| Events | **No schedule endpoint exists.** `/v2/events` and `/v1/event_names` → 503. `/v1/event_details` (static, legacy) and `/v2/worldbosses`, `/v2/mapchests` (ids only) exist. | — | Schedules (world bosses, metas) must be a **curated dataset** maintained by us (source: GW2 wiki timers, CC BY-NC-SA → attribution needed). |

GW2 API rate limit: per IP/key token bucket (~300 burst, refill ~5 req/s).
A full price snapshot (140 requests) fits easily.

**Neon Free plan** (neon.com/pricing): 1 GB storage per project, 100 CU-hours
per project/month, compute auto-suspends after 5 min idle, 5 GB egress.

Price-history volume estimate (all 28k items, ~60 B/row incl. overhead):

| Interval | Rows / day | Storage / month |
|----------|-----------|-----------------|
| 5 min | 8.1 M | ~14 GB |
| 15 min | 2.7 M | ~4.8 GB |
| 1 h | 672 k | ~1.2 GB |
| 1 h, only changed prices | ~150–300 k (est.) | ~0.3–0.5 GB |
| daily OHLC aggregate | 28 k | ~50 MB |

⇒ Neon Free (1 GB, 100 CU-h, any cron < 5 min apart keeps compute awake ⇒
~180 CU-h/month at 0.25 CU) **cannot hold full-resolution history** and a
frequent cron exhausts compute hours. Needs retention/downsampling or a
different host.

Hosting alternatives (free tiers, to be verified at decision time):

| Option | What you get | Fit |
|--------|--------------|-----|
| Neon Free + separate app host | 1 GB Postgres, suspends | DB only; needs downsampling; app host still needed |
| Supabase Free | 500 MB Postgres, pauses after 7 days inactivity | smaller than Neon |
| Oracle Cloud Always Free VM (ARM, up to 4 OCPU / 24 GB RAM, 200 GB disk) | full VM: run backend + Postgres (+ TimescaleDB compression) + cron | best capacity, more ops work (updates, backups) |
| Cloudflare R2 (10 GB free) / Pages | static files with CDN, zero egress fees | ideal for serving snapshots to addons |
| Koyeb / Render free web service | small container, may sleep | app host only |

## Proposed stack (for decision)

- **Runtime/web:** `tokio` + `axum` (de-facto standard, tower middleware).
- **DB access:** `sqlx` with Postgres and compile-time checked queries
  (`query!`/`query_as!`) + `sqlx migrate`.
- **HTTP client:** `reqwest` (+ `backon` or own retry) for the GW2 API.
- **Scheduling:** in-process `tokio-cron-scheduler` *or* external trigger
  (GitHub Actions `schedule`) — open question.
- **Serialization/types:** `serde`; API DTOs in `crates/trailpack-api-types`
  shared with the addons (end-to-end typing, like tRPC in TS).
- **API docs:** `utoipa` (OpenAPI generated from types).
- **Rate limiting:** `tower_governor` (per-IP token bucket).
- **Observability:** `tracing` + `tracing-subscriber`.
- **Config:** typed config struct via `figment` or `envy` (env vars).
- **Errors:** `thiserror` in library code, `anyhow` only in `main`.

## Security model (proposal, to grill)

A secret shipped inside a public DLL **cannot stay secret** — anyone can read
it from the binary or from the HTTP request. So "only my addons can call the
API" is not enforceable; we can only raise the bar and limit damage:

1. Serve all non-personal data as **static, versioned snapshots** (JSON/
   compressed) behind a CDN with ETags → abuse costs us nothing.
2. Dynamic endpoints (price history queries) behind **per-IP rate limits**,
   request size limits, timeouts, and Cloudflare (WAF/bot protection).
3. Optional "client identity" header (addon name + version) for analytics and
   blocking old versions — explicitly *not* a security boundary.
4. Database credentials and admin endpoints never exposed; admin via separate
   auth (or none at all — cron runs inside the server).

## Open questions

- ~~Event data format~~ → TOML (data-pipeline D23).
- ~~Waypoints scope~~ → all waypoints (data-pipeline D20).

## Decisions

| # | Decision | Date |
|---|----------|------|
| B1 | Price history for **all tradeable items**, **one daily value per item** (no hourly data), rolling **1-year retention** (rows older than 365 days deleted daily) — enables yearly comparisons such as Halloween | 2026-10-31 |
| B2 | Event schedules are a curated dataset in `data/events/` (source: GW2 wiki; currently via Wayback Machine because the wiki is behind a login — see `data/events/README.md`) | 2026-10-31 |
| B3 | Security: no "secret" client key; public data as static versioned files behind a CDN, dynamic endpoints rate-limited per IP | 2026-10-31 (pending CDN explanation) |
| B4 | Mirrored languages: en + de only | 2026-10-31 |
| B5 | Prices fetched **once per day** (no intraday refresh) to minimize load | 2026-10-31 |
| B6 | Stay provider-agnostic: plain Postgres only, `sqlx` migrations, connection via `DATABASE_URL`; migration = `pg_dump`/`pg_restore` + env change; monitor DB/disk size and warn at 80 % (the original "start on Neon" part is superseded by B16) | 2026-10-31 |
| B7 | Backend sync of custom events between users → backlog | 2026-10-31 |
| B8 | From day one: **always-on Rust runtime with REST API + database** (no static-files-only setup) | 2026-10-31 |
| B9 | One sync run per day at **00:30 UTC** (prices, items, achievements, waypoints, events). Manual re-sync is **admin-only** (never triggerable by addon users), no cooldown | 2026-10-31 |
| B10 | Server binary runs as Docker image (built in CI), deployed via SSH script **only with explicit user approval**; Caddy as reverse proxy/TLS; secrets only as env vars on the server | 2026-10-31 |
| B11 | REST API versioned under `/v1`, `?lang=en|de`; endpoints: `/v1/events`, `/v1/waypoints`, `/v1/achievements`, `/v1/items` (**all at once** or `?ids=`), `/v1/prices` (**all at once** or `?ids=`), `/v1/prices/{item_id}/history`, `/v1/meta`; OpenAPI via `utoipa`; DTOs in `crates/trailpack-api-types` shared with core | 2026-10-31 |
| B12 | Admin re-sync via `POST /admin/sync` protected by an admin token only the owner holds (never shipped in addons), strict rate limit, no cooldown | 2026-10-31 |
| B13 | Domain bought at **INWX** at the first public release (~5 €/year, DNS included); subdomains (e.g. `api.`, `www.`, further sites) via DNS records, routed per subdomain by Caddy with automatic TLS; name/TLD decided before release. | 2026-10-31 |
| B20 | **No Cloudflare / no US providers.** DNS at INWX, DDoS protection by Hetzner, rate limiting in the Rust server; a CDN only when needed, then Bunny.net (EU). Independent of the pending CJEU ruling on the EU-US Data Privacy Framework (C-703/25 P). Supersedes the CDN part of B3 | 2026-10-31 |
| B21 | No backups at launch (DB holds only public, re-syncable data except price history) → backlog BL4 | 2026-10-31 |
| B19 | Hosting roadmap: phases 0–2 run **locally in WSL** (`docker compose`: Postgres + server, addon talks to localhost, 0 €); from the first public trailpack-core release on: **Hetzner Cloud, Germany** (CX23 or CAX11, whichever is cheaper); migration-ready via Docker + `pg_dump` + DNS switch | 2026-10-31 |
| B14 | Caching inside the Rust process (`ArcSwap` for daily data sets, `moka` for price histories); no Redis/Elasticsearch until a concrete need (e.g. multiple servers) | 2026-10-31 |
| B15 | Bulk item endpoint contains only addon-relevant fields: id, name, icon URL, rarity, type, level, chat link; extended on demand | 2026-10-31 |
| B16 | **No Neon.** Postgres runs in Docker on our own server; code stays provider-agnostic (`DATABASE_URL`, `sqlx` migrations); nightly off-site backups | 2026-10-31 |
| B17 | **GDPR (DSGVO) compliance is mandatory** for every component (hosting, backups, logs, CDN, websites) | 2026-10-31 |
| B18 | GDPR measures: IPs in logs truncated (last octet removed), logs deleted after 7 days; rate-limit counters in memory only; no tracking/analytics; every website has Impressum + privacy policy; addons send only addon name + version (no account/character data) | 2026-10-31 |
