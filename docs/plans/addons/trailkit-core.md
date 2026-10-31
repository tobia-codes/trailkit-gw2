# trailkit-core

- **Status:** draft (grilling round 2 pending)
- **Crate path:** `addons/trailkit-core`
- **Nexus signature / ID:** TBD (negative random during development)

## Goal

The hub addon of the trailkit family. It fetches, caches and provides data
(from the trailkit static data files — see
[`data-pipeline.md`](../data-pipeline.md) — and — with a player API key — from the official
GW2 API) to the other trailkit addons through Nexus, so each addon stays
small and the backend load stays minimal.

## Requirements (from user)

- Provides data to other addons via the Nexus interface (DataLink / events).
- **Caching is essential**: minimize downloads from the data host; only
  refetch when data is actually stale. Cache validity per data type must be
  defined (see Open questions).
- Background refresh on a fixed interval without user action — and without
  stutter (see `nexus-addon-dev` skill §2).
- **GW2 API key** can be stored; enables player-specific data. Security is
  paramount (DPAPI encryption, never sent anywhere but the GW2 API, masked in UI).
- Settings let the user choose **which data sets are fetched**. If a data set
  another addon needs is disabled, that addon shows a guided hint explaining
  what to enable (and a button to jump there).
- Own settings window, help page, en/de, dark/light theme (project-wide rules).

## Technical design (proposal)

- **Worker thread** (one): scheduler loop with `recv_timeout(next_due)`,
  blocking `ureq` HTTP, writes disk cache, publishes `ArcSwap` snapshots.
- **Disk cache** in `Paths_GetAddonDirectory("trailkit/core")`: one file per
  data set + metadata (`fetched_at`, `etag`, `schema_version`, `lang`).
  Startup loads from disk first → addons have data instantly, even offline.
- **Conditional requests** (`If-None-Match` / ETag) to the data host;
  `meta.json` first, then only data sets whose hash changed → most
  refreshes are a 304 with no body.
- **Provider API** for other addons: versioned `#[repr(C)]` service table
  shared via DataLink `TRAILKIT_CORE_V1`, typed safe wrapper in
  `crates/trailkit-protocol` (`CoreClient`). Change notifications via Nexus
  event `TRAILKIT_CORE_DATA_CHANGED` (payload: data set id + generation).
- **Data set registry**: each data set (events, waypoints, items, prices,
  achievements, account-*) has id, required API key permissions, default
  enabled flag, refresh policy. Consumers declare which data sets they need;
  core can therefore tell the user which addon needs which data set.
- **Theme & language hub** (proposal): core owns the shared theme file and
  language preference so all addons stay in sync — falls back to local
  defaults when core is absent.

## Cache validity (draft — to grill)

| Data set | Source | Proposed refresh | Rationale |
|----------|--------|------------------|-----------|
| Event schedules | data files | on start + every 24 h, ETag | changes with patches only |
| Waypoints | data files | on start + 7 d, ETag | rare changes |
| Achievements (defs) | data files | on start + 24 h, ETag | patch-dependent |
| Items (defs) | data files | full file per active language when `meta.json` hash changes (C9) | changes on patches |
| Prices | data files | daily (one snapshot per day, D-series) | only a daily snapshot exists |
| Account data (wallet, achievements progress, …) | GW2 API w/ key | on demand + 5 min min interval | per-player, API caches ~5 min anyway |
| Invalidation trigger | game build id from MumbleLink | on new build → revalidate static sets | patches change defs |

## Open questions

- Exact cross-addon protocol (service table vs. events vs. shared files).
- Item data: full mirror on disk (~tens of MB) vs. lazy per-id fetch?
- None currently (round 13 settled C6–C13).

## Decisions

| # | Decision | Date |
|---|----------|------|
| C1 | API key in v1: store (DPAPI), validate via `/v2/tokeninfo`, show account name + permissions only; more account data sets only when an addon needs them | 2026-10-31 |
| C2 | Shared theme + language live in a shared file under `addons/trailkit/shared/`, readable by every trailkit addon without core; changes broadcast via Nexus event | 2026-10-31 |
| C3 | Stores custom events (see trailkit-event E4) and exposes create/update/delete via the protocol | 2026-10-31 |
| C4 | Bulk items/prices downloaded only in the active language; other language fetched on language switch | 2026-10-31 |
| C5 | Refresh on game start, then daily with random 0–60 min jitter after the data job (00:17 UTC, catch-up 06:17 UTC — data-pipeline D7); missed refreshes caught up on next start; ETag → 304 when unchanged | 2026-10-31 |
| C6 | Cross-addon protocol: versioned `#[repr(C)]` service table via DataLink `TRAILKIT_CORE_V1` + Nexus event `TRAILKIT_CORE_DATA_CHANGED` (data set id + generation); safe wrapper `CoreClient` in `crates/trailkit-protocol` | 2026-10-31 |
| C7 | Compatibility: functions are only appended, never changed; consumers check the table version; core too old → hint "please update trailkit-core" | 2026-10-31 |
| C8 | Data host unreachable: keep using the disk cache; retries after 1/5/15/60 min, then hourly; when data is **older than 48 h** a visible "data outdated (from <date>)" message is shown (toast once per session + persistent badge in core and in affected addons, no blocking popup; confirmed in round 17) | 2026-10-31 |
| C9 | Items: full file per active language stored compressed on disk (~4 MB); loaded into memory (compact representation, est. 6–8 MB) only when the first consumer needs items; dropped from memory when the last consumer releases the data set or it is disabled | 2026-10-31 |
| C10 | A data set is downloaded only if enabled in settings **and** at least one installed addon declared it | 2026-10-31 |
| C11 | First-start notice (data-pipeline D14) is a modal in core; until confirmed, consumer addons show "data downloads not enabled in trailkit-core" with a button that opens it | 2026-10-31 |
| C12 | Game build change (MumbleLink) → core re-checks `meta.json` immediately | 2026-10-31 |
| C13 | All public data sets work without an API key; the key only unlocks account data (C1) | 2026-10-31 |
