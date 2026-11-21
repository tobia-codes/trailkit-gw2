# trailkit backend

- **Status:** **deferred** (2026-10-31, see [`data-pipeline.md`](data-pipeline.md) D1), except the
  addon status endpoint, which must be live before the first addon release (B22, B23). Kept as the
  target design for when a dynamic feature (e.g. price history) needs a server.
  Data formats and paths of the static files are chosen so this server can
  take over without changing core.
- **Crate path:** `services/trailkit-api/` (binary, skeleton since R90) + shared types in
  `crates/trailkit-model` (R75); migrations and `Dockerfile` in the crate, server-wide
  deploy configuration in the root `deploy/` (R90)

## Goal

A Rust service that mirrors the data the trailkit addons need from the
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
- Security: rate limiting; ideally only trailkit addons can query the API.
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

## Proposed stack (for decision; not grilled yet, see open questions)

- **Runtime/web:** `tokio` + `axum` (de-facto standard, tower middleware).
- **DB access:** `sqlx` with Postgres and compile-time checked queries
  (`query!`/`query_as!`) + `sqlx migrate`.
- **HTTP client:** `reqwest` (+ `backon` or own retry) for the GW2 API.
- **Scheduling:** in-process `tokio-cron-scheduler` *or* external trigger
  (GitHub Actions `schedule`) — open question.
- **Serialization/types:** `serde`; response types are `crates/trailkit-model`
  types shared with the addons (end-to-end typing, like tRPC in TS; R75).
- **API docs:** `utoipa` (OpenAPI generated from types).
- **Rate limiting:** `tower_governor` (per-IP token bucket).
- **Observability:** `tracing` + `tracing-subscriber`.
- **Config:** typed config struct via `figment` or `envy` (env vars).
- **Errors:** `thiserror` in library code, `anyhow` only in `main`.

## Security model (proposal; superseded where B20, B43 differ)

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
- ~~Addon status (B23) open points~~ → B24.
- **Tech stack:** the proposed stack below is not grilled yet. Grill it (web framework, DB access,
  migrations, deployment, monitoring, GDPR) and where shared strings such as the lock texts live
  (`trailkit-i18n`, ROADMAP layout) before the B23 server work starts.
- ~~R121 round (sessions 41–42)~~ → B26–B44, R132.
- ~~Price poll frequency~~ → B39.
- **In-game problem reports (B25), open points:**
  - Q288: where the flag lives: `/v1/meta` (the user's first idea) or the `/v1/addons/status` response,
    which every addon already shares through `update-status.json` (one request, B23 Q228).
  - Q289: what the server does with a report: store it in Postgres for an admin view, create a GitHub
    issue (public or private repository), or both.
  - Q290: data and GDPR (rule 10): fields, log size limit, retention, consent text, an optional contact
    address for replies (personal data), `PRIVACY.md` update.
  - Q291: abuse protection: per-IP rate limit and size limits (B3, B18), spam handling.
  - Q292: flag granularity: one global switch or per addon (and per version).
  - Q293: form design and how the user learns about a reply (needs visual approval, R87).
- **Sync of the note tree and the trash** (Q572, 2026-11-21, raised with memo M218; with B26, B37,
  B45): B26/B37 cover notes and settings only. Open: how groups, the order of children and the
  trash are synced (one encrypted tree document, or per group or note with its own version); whether
  the trash is synced, with its `child_order` snapshots (memo M218), or stays per device; and what
  happens when two devices change the structure at once (one restores a note from a trashed group
  while the other deletes that group for good; both reorder the same group). Principle so far: the
  server never orders or repairs anything (B28, ciphertext only); the client computes the new tree,
  encrypts it and uploads it, and the web frontend decrypts the same arrays.
- **Tree logic in the web frontend** (Q572): the frontend needs the same tree and trash rules as memo
  (restoring with the group's shell, `child_order`, skipping missing entries). Recommendation: compile
  `note_tree` of `trailkit-memo-shared` to WebAssembly for the browser, one implementation with the
  same tests, instead of a TypeScript copy. To grill with the frontend (B42).
- **Notes table under end-to-end encryption** (Q573, proposal from 2026-11-21, with B28, B30, B37, B45):
  the server stores only what it must know, the content is one encrypted block: `id`, `owner` (B45),
  `version` (B37), `format_version` (B30), `ciphertext` + `nonce`, `updated_at`; a second table holds the
  wrapped note keys per authorized account (B30). No content columns, so the server cannot search, sort
  or filter; extension tables (B45) only for data the server must see (e.g. which guild has access).
  Metadata it still sees (number of notes, rough size, change times) goes into `PRIVACY.md` (rule 10).
- **Database** (Q574, part of the tech stack grilling): recommendation Postgres, encrypted notes as
  `bytea`, flexible data as `JSONB` where needed; a document database gains nothing on ciphertext,
  and a second database doubles operations, backups and GDPR work. Alternatives: a document database
  for everything, or both.
- **Marketplace versus private notes** (with BL27, B26, B30): published notes are a deliberate,
  separate step: the addon decrypts the note locally and uploads a **plaintext public copy** to its own
  table (title, text, category, language, author, counts), so search, categories and moderation work;
  the private note stays encrypted. A download becomes the downloader's own note, encrypted with their
  key when synced. Sharing with chosen accounts (B30) stays encrypted with wrapped keys.

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
| B11 | REST API versioned under `/v1`, `?lang=en|de`; endpoints: `/v1/events`, `/v1/waypoints`, `/v1/achievements`, `/v1/items` (**all at once** or `?ids=`), `/v1/prices` (**all at once** or `?ids=`), `/v1/prices/{item_id}/history`, `/v1/meta`; OpenAPI via `utoipa`; DTOs in `crates/trailkit-api-types` shared with core (**R75:** responses use `crates/trailkit-model`) | 2026-10-31 |
| B12 | Admin re-sync via `POST /admin/sync` protected by an admin token only the owner holds (never shipped in addons), strict rate limit, no cooldown | 2026-10-31 |
| B13 | Domain bought at **INWX** (timing/name superseded by ROADMAP R27: trailkit.de, bought now) (~5 €/year, DNS included); subdomains (e.g. `api.`, `www.`, further sites) via DNS records, routed per subdomain by Caddy with automatic TLS; name/TLD decided before release. | 2026-10-31 |
| B20 | **No Cloudflare / no US providers.** DNS at INWX, DDoS protection by Hetzner, rate limiting in the Rust server; a CDN only when needed, then Bunny.net (EU). Independent of the pending CJEU ruling on the EU-US Data Privacy Framework (C-703/25 P). Supersedes the CDN part of B3 | 2026-10-31 |
| B21 | No backups at launch (DB holds only public, re-syncable data except price history) → backlog BL4 | 2026-10-31 |
| B19 | Hosting roadmap: phases 0–2 run **locally in WSL** (`docker compose`: Postgres + server, addon talks to localhost, 0 €); from the first public trailkit-core release on: **Hetzner Cloud, Germany** (CX23 or CAX11, whichever is cheaper); migration-ready via Docker + `pg_dump` + DNS switch | 2026-10-31 |
| B14 | Caching inside the Rust process (`ArcSwap` for daily data sets, `moka` for price histories); no Redis/Elasticsearch until a concrete need (e.g. multiple servers) | 2026-10-31 |
| B15 | Bulk item endpoint contains only addon-relevant fields: id, name, icon URL, rarity, type, level, chat link; extended on demand | 2026-10-31 |
| B16 | **No Neon.** Postgres runs in Docker on our own server; code stays provider-agnostic (`DATABASE_URL`, `sqlx` migrations); nightly off-site backups | 2026-10-31 |
| B17 | **GDPR (DSGVO) compliance is mandatory** for every component (hosting, backups, logs, CDN, websites) | 2026-10-31 |
| B18 | GDPR measures: IPs in logs truncated (last octet removed), logs deleted after 7 days; rate-limit counters in memory only; no tracking/analytics; every website has Impressum + privacy policy; addons send only addon name + version (no account/character data) | 2026-10-31 |
| B22 | **Backend before the first addon release; forced updates.** The backend must exist before any addon is published (supersedes the "deferred" status for this feature). Reason: the owner must be able to lock an addon version after a breaking change until the user has updated. Design: B23. | 2026-11-07 |
| B23 | **Addon status endpoint** (user decisions Q222–Q237, 2026-11-10; amends B18, B19). **Scope (Q222):** at the first addon release only a minimal Rust server (B8) with `GET /v1/addons/status` and `/v1/meta`; the B11 data endpoints follow with core. **Hosting (Q223, Q231):** everything stays local (docker compose, B19) until the user really releases; then Hetzner Germany at `api.trailkit.de`. Debug builds ask `http://localhost:<port>`, release builds `https://api.trailkit.de`; until the release the check fails silently (open). **Model (Q224, Q232, Q233):** per addon in Postgres `minimum_version` and `state` = `Active` or `Locked { reason: Maintenance \| Problem }`, extensible with new reasons; a lock applies to all versions (single versions are excluded via `minimum_version`). **Message (Q234):** a lock carries an optional expected end (UTC, shown in local time) and an optional plain-text message in en and de (two fields, ≤ 200 characters); without a message in the user's language the addon shows the localized default "temporarily locked" / "temporarily under maintenance". **Admin (Q229):** `PUT /admin/addons/{addon}` with the admin token (B12) sets minimum version and lock, effective without a deploy. **Request (Q228, Q230):** one request returns all addons; no parameters, no addon name or version, neutral user agent; the server treats only the IP per B18. Result and timestamp go to the shared `trailkit/shared/update-status.json`; an addon reads it first and asks only when it is older than the freshness limit (≈ 9 min), so all addons share one request. **When (Q225, Q235):** at load in a background thread, then every 60 min while unlocked and every 10 min while locked, so a lock ends by itself. **Offline (Q226):** unreachable server → the cached result applies; without a cache the addon is unlocked (fail open). **Locked (Q227, Q237):** for update, maintenance and problem alike the addon shows only a notice window where its main window would appear (never opens by itself), no data access or writes. The update lock wins over the others because the user can fix it. Update notice approved (R87): https://claude.ai/artifact/EDHSjZGj6aLW3FXsPMHLDb | 2026-11-10 |
| B24 | **Addon status locks, details** (user decisions Q236–Q241, 2026-11-11; amends B23). **Offline expiry (Q236):** with the server unreachable, a cached maintenance or problem lock ends at its expected end, without one at the latest 24 h after the server last confirmed it; a cached update lock stays, because the user can lift it by updating. **Nexus behavior (checked in source v2026.11.8.830):** Nexus checks for updates only when it loads an addon (game start or manual load in the addon list) and then hot-reloads it without a restart; there is no periodic check. So the update lock appears mainly when the minimum version rises mid-session or the download fails. **Update hint (Q238):** "Nexus installs the update when {addon} is loaded again. Restart the game or reload {addon} in the Nexus addon list." / "Nexus installiert das Update, sobald {addon} neu geladen wird. Starte das Spiel neu oder lade {addon} in der Nexus-Addonliste neu." **Maintenance and problem lock (R87, approved 2026-11-11):** same window as the update lock; maintenance uses icon `wrench` (new in the icon font) in `info`, problem uses `triangle-alert` in `warning` (not `danger`). Titles "Maintenance" / "Wartungsarbeiten" and "Temporarily locked" / "Vorübergehend gesperrt"; the operator message replaces the default sentence "{addon} is temporarily under maintenance." / "{addon} is temporarily locked because of a known problem."; then the addon-specific data sentence and the hint "{addon} unlocks automatically once the lock is lifted." **Expected end (Q241):** one sentence in `text.secondary`, "Expected until <time>." / "Voraussichtlich bis <time>.", time via the existing `format_local_time`, never broken inside. Mockup: https://claude.ai/artifact/GrHi26PB5b7ggo5fKKtsAx | 2026-11-11 |
| B25 | **In-game problem reports, switched by the backend** (user decision 2026-11-11; amends R95, B23; R25 still holds: nothing is sent without a user action). The help window's problem page gets a second variant: a form that sends the description and the exported log straight to the backend, so the user needs neither a GitHub account nor to find the log. A **feature flag from the backend** decides which variant the addons show; the owner switches it without a deploy. Server unreachable, no cached answer or flag off → the R95 variant (GitHub issue form link plus log export). R95 ships first unchanged; the form comes with the backend. Sending only on an explicit click after a consent notice. Open points: Q288–Q293 | 2026-11-11 |
| B26 | **Opt-in sync of addon data** (user decision Q463, 2026-11-16; part of R121): at the first joint release (R114) accounts can sync **memo notes and event favorites/settings** only; sync is opt-in, and without login nothing leaves the PC. `PRIVACY.md`/`PRIVACY.de.md` describe the sync and its end-to-end encryption in the same change (R99) | 2026-11-16 |
| B27 | **Account recovery without email** (user decision Q464, 2026-11-16): at sign-up the user gets one-time **recovery codes**; no GW2 API key or account name is used for recovery. Without password and codes the account is lost. A device that is still logged in can set a new password without a code. The recovery codes also unlock the end-to-end key (B28) | 2026-11-16 |
| B28 | **End-to-end encryption of synced notes** (user decision Q465, 2026-11-16; follow-ups Q466): the addon and the browser encrypt; the server and admins see ciphertext only. Proposed model (not yet settled in detail): a random X25519 account key pair; the private key is stored on the server only wrapped twice (password-derived key via Argon2id, recovery-code-derived key), so a password change only rewraps it; a random key per note (XChaCha20-Poly1305) wrapped per authorized account, which also enables sharing later. Login uses a value derived separately from the password, never the password itself. Matrix serves only as a model for key backup, recovery key and key verification, not Megolm. Only audited libraries, no own cryptography. The **frontend states openly** that the addons encrypt notes end to end. Diagram: https://claude.ai/artifact/GRKXsoSC4uTkGcDk6UXtLq | 2026-11-16 |
| B30 | **Sharing prepared, not built** (user decision Q466, 2026-11-16; amends B28): the key pair, the key per note and wrapped note keys stored as a list per note (one entry per authorized account) ship at the first release, plus a format version field on every encrypted record, so sharing needs no data migration later. The sharing UI (find a user, rights, revoke with key rotation, per note or per folder) goes to the backlog | 2026-11-16 |
| B29 | **Two-factor authentication** (user wish 2026-11-16): accounts get 2FA, presumably TOTP (authenticator app; passkeys are hard in the addon without a browser). **Mandatory for admins and moderators, optional for all other users** (user decision Q467, 2026-11-16) | 2026-11-16 |
| B31 | **Login window in the addon** (user decision Q468, 2026-11-16): the addon has its own login window (user name, password, 2FA code when enabled, B29) and derives the keys itself (B28). Pairing through the browser with a code stays possible later; the key model allows it. The window's design needs an R87 mockup | 2026-11-16 |
| B32 | **Addon stays logged in** (user decision Q470, 2026-11-16): the addon keeps the private key and the session token in a file in its own folder, protected with Windows DPAPI (current Windows user only). "Stay logged in" is **on by default**; users on a shared PC can turn it off, then the key lives in memory only until the game closes. The addon only ever decrypts its own file, never other applications' data | 2026-11-16 |
| B33 | **Backups from the first release** (user decision Q471, 2026-11-16; supersedes B21, which assumed only re-syncable public data): with accounts, encrypted notes and the forum, the database is backed up **nightly**, encrypted with a key only the owner holds, to EU storage (provider: Q472). Retention: 7 daily and 4 weekly copies; deleted accounts disappear from backups once retention runs out (about 5 weeks), stated in `PRIVACY.md`/`PRIVACY.de.md` (R99) | 2026-11-16 |
| B34 | **Backup storage: Scaleway Object Storage** (user decision Q472, 2026-11-17; completes B33): France, billed per GB, a different provider than the Hetzner server, so backups survive a Hetzner outage or account block. Check current prices and sign the DPA (AVV) at setup; `PRIVACY.md`/`PRIVACY.de.md` name the provider (R99) | 2026-11-17 |
| B35 | **Browser session length** (user decision Q476, 2026-11-17): 30 days sliding (each use extends it), cookie `HttpOnly`, `Secure`, `SameSite=Lax`; an option "Stay signed in on this device", without it the session ends with the browser; "Sign out all sessions" in the account settings. The addon keeps B32 | 2026-11-17 |
| B36 | **User names** (user decision Q477, 2026-11-17): the user name is the login (no email); unique, 3–24 characters `a–z`, `0–9`, `_`, `-`; login ignores case, display keeps it; renaming at most every 30 days; reserved names such as `admin` or `trailkit` are blocked; no GW2 account name required | 2026-11-17 |
| B37 | **Sync conflicts** (user decision Q478, 2026-11-17; with B26, B28): the server cannot merge ciphertext, so every note has a version number and the server accepts a change only on the current base version; on a conflict the client keeps both: the second copy becomes "Note (conflict device X)", nothing is lost. Favorites and settings: last write wins per field | 2026-11-17 |
| B38 | **Roles** (user decision Q479, 2026-11-17): three fixed roles: **admin** (everything, grants roles), **moderator** (moderates the community area, blocks users) and **user**; admin and moderator need 2FA (B29); every admin/moderator action goes to an audit log; finer permissions only on real need | 2026-11-17 |
| B39 | **Price history resolution** (user decision Q473, 2026-11-17; amends B1, retention stays 12 months): every **5 minutes for the last 7 days**, **hourly for 12 months**, older rows deleted; about 20 GB, downsampled automatically by the database. Measured on 2026-11-17 for comparison (item 19721): DataWars2 daily values since 2012-10 and hourly for about 8 days; GW2BLTC 6-hour values since 2021-12, hourly for about 5 weeks, about 4-minute values for about 2 days | 2026-11-17 |
| B40 | **Discord instead of an own forum at the start** (user decision Q480, 2026-11-17; amends R121): a free Discord server (boosts are cosmetic only) for community, questions and wishes; the website only links to it (no widget or embed, so our pages send nothing to Discord); problem reports without an account stay the B25 form. Trade-offs accepted: Discord is a US provider, needs an account with email, is not searchable. The own forum goes to the backlog and is reassessed when the community grows; the moderator role (B38) then moderates Discord | 2026-11-17 |
| B41 | **API host with the product name** (user decision Q482, 2026-11-17; amends B23 and D19): the API lives at **`api.gw2.trailkit.de/v1/`** (one subdomain per product line like R52, so every backend can run on its own server via DNS). After the first backend release the addons get their data from it; GitHub stays an archive; the switch goes through `meta.json` without an addon update | 2026-11-17 |
| B42 | **Frontend framework: TanStack Start** (user decision Q483, 2026-11-17; R121): type-safe routes, params, search params and server functions (rule 8), TanStack Query built in, rendering chosen per route (Q483b); self-hosted as a Node container next to the Rust server, which stays the only source of truth | 2026-11-17 |
| B43 | **API keys without an extractable secret** (user decisions Q481, 2026-11-17; amends B3, B18): no API use without a key. **Addons (Q481c):** a fixed client key per addon (not a secret: it can be read from the DLL; it identifies addon and version and is rotated with an addon update), plus a per-IP rate limit and a cache in front of the public data; per-installation tokens with proof of work only if real abuse appears. **Website (Q481d):** the browser never calls `api.gw2.trailkit.de` directly, only the TanStack Start server functions (backend for frontend), which alone hold a secret API key; limits per session or IP. Third parties get own keys on request in their account later. Public read endpoints for events, prices and items, private endpoints for sync, account and community need a session; OpenAPI documentation, unofficial for third parties at first | 2026-11-17 |
| B44 | **Rendering per route** (user decision Q483b, 2026-11-17; with B42): public pages (addons, help, imprint, privacy) are **prerendered at build time**; account, notes, sync and admin render **in the browser** (end-to-end encrypted notes cannot be rendered on the server anyway); the Node server only runs the server functions (B43); real SSR only later and only where needed, e.g. item and price pages for search engines. Estimated cost: about 60–120 MB RAM for Node, negligible CPU | 2026-11-17 |
| B45 | **Notes data model** (user decision 2026-11-18, memo M200–M203): one `notes` table for all notes with an owner column (account or, later, guild); fields of a specific note sort (e.g. guild notes) go into a 1:1 extension table such as `guild_note_details` keyed by note id, never into a separate copy of the notes table. Ownership values are `snake_case` text with a CHECK constraint, not a database enum | 2026-11-18 |
