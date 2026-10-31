# trailkit-datagen

Command line tool that captures the daily Guild Wars 2 trading post **price
archive** (phase 0b). Later it will also generate the static data files that
trailkit-core downloads. Plan and decisions:
[`docs/plans/data-pipeline.md`](../../docs/plans/data-pipeline.md).

It only reads the public GW2 API (no API key) and writes into a local
directory. It never talks to GitHub: the scheduled workflow creates the
monthly release `archive-YYYY-MM` and uploads the files (D34).

## Build and test

```sh
cargo build -p trailkit-datagen --release
cargo test -p trailkit-datagen     # uses committed fixtures, never the live API (D36)
```

## Usage

```sh
# Today's archive (UTC). Skips data sets the day's manifest in --out-dir
# already lists as complete, so the same command is the daily run and the
# catch-up run (put the day's manifest into --out-dir first).
trailkit-datagen archive --out-dir out/
trailkit-datagen archive --out-dir out/ --fail-if-incomplete   # catch-up: exit 1 if gaps remain

# WvW snapshot shortly before a weekly reset (D29)
trailkit-datagen wvw-snapshot --region eu --out-dir out/
```

Log level via `RUST_LOG` (default `info`). A full archive run takes about
4–5 minutes (throttled to ~4 requests/s, D26).

## Files per day

| File | Content | Size (gzip, 2026-10-31) |
|------|---------|------|
| `<date>.prices.ndjson.gz` | `/v2/commerce/prices` for every tradeable item, one API object per line, unchanged except whitespace removed | ~0.4 MB |
| `<date>.orderbook.ndjson.gz` | `/v2/commerce/listings` condensed (D17): per side best 5 tiers, tier count, total listings/quantity, depth within 1/5/10 % | ~1.9 MB |
| `<date>.wvw.ndjson.gz` | `/v2/wvw/matches?ids=all`, one match per line | ~25 KB |
| `<date>.gem-exchange.json.gz` | 9 gem exchange quotes at the in-game quantities (D31), raw responses with the amount paid | < 1 KB |
| `<date>.manifest.json` | runs (start/end, tool version, game build) and per data set `status` (`complete` / `partial` + `failed_ids` / `failed` + `error`), `captured_at`, records, size, SHA-256 | ~2 KB |
| `<date>.wvw-<eu\|na>-reset.ndjson.gz` | WvW snapshot before the weekly reset (Fri/Sat only) | ~25 KB |

Failure handling (D35): each request is retried up to 5 times with
exponential backoff; a failed data set does not stop the others; the
catch-up run repeats every data set that is not `complete`.

## Postgres import (sketch)

All timestamps are UTC RFC 3339, money is integer copper. Planned target
tables for the later backend (load the NDJSON via `COPY` into a staging
`jsonb` table, then insert typed rows):

```sql
-- <date>.prices.ndjson.gz
CREATE TABLE price_snapshot (
    item_id         integer     NOT NULL,
    captured_at     timestamptz NOT NULL,  -- manifest datasets.prices.captured_at
    whitelisted     boolean     NOT NULL,
    buy_quantity    integer     NOT NULL,
    buy_unit_price  integer     NOT NULL,
    sell_quantity   integer     NOT NULL,
    sell_unit_price integer     NOT NULL,
    PRIMARY KEY (item_id, captured_at)
);

-- <date>.orderbook.ndjson.gz (one row per item and side)
CREATE TABLE orderbook_snapshot (
    item_id             integer     NOT NULL,
    captured_at         timestamptz NOT NULL,
    side                text        NOT NULL CHECK (side IN ('buy', 'sell')),
    best_tiers          jsonb       NOT NULL,  -- [{listings, unit_price, quantity}], best first
    tier_count          integer     NOT NULL,
    total_listings      bigint      NOT NULL,
    total_quantity      bigint      NOT NULL,
    depth_1_percent     bigint      NOT NULL,
    depth_5_percent     bigint      NOT NULL,
    depth_10_percent    bigint      NOT NULL,
    PRIMARY KEY (item_id, captured_at, side)
);

-- <date>.wvw.ndjson.gz and the reset snapshots
CREATE TABLE wvw_match_snapshot (
    match_id    text        NOT NULL,      -- e.g. "2-1"
    captured_at timestamptz NOT NULL,
    match       jsonb       NOT NULL,      -- full API object
    PRIMARY KEY (match_id, captured_at)
);

-- <date>.gem-exchange.json.gz (one row per quote)
CREATE TABLE gem_exchange_quote (
    captured_at   timestamptz NOT NULL,
    direction     text        NOT NULL CHECK (direction IN ('gems_to_coins', 'coins_to_gems')),
    paid          bigint      NOT NULL,    -- gems or copper
    coins_per_gem integer     NOT NULL,
    quantity      bigint      NOT NULL,    -- copper or gems received
    PRIMARY KEY (captured_at, direction, paid)
);
```

## Layout

| Path | Content |
|------|---------|
| `src/main.rs` | CLI (`clap`) |
| `src/archive.rs` | daily run: which data sets, capture, manifest updates |
| `src/manifest.rs` | manifest types (`Dataset`, `DatasetOutcome`, `Run`) |
| `src/gw2_api.rs` | throttled, retrying `ureq` client |
| `src/orderbook.rs` | order book condensation (D17) |
| `src/gem_exchange.rs`, `src/wvw.rs` | the smaller data sets |
| `src/raw_json.rs`, `src/compact_json.rs` | lossless array → NDJSON lines |
| `src/archive_file.rs` | gzip writer, size + SHA-256 |
| `fixtures/` | small API responses for the tests |
