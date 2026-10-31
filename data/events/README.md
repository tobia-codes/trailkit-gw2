# Event schedules (curated)

The official Guild Wars 2 API has **no event schedule endpoint**
(`/v2/events` and `/v1/event_names` return 503). Schedules for world bosses and
meta events are therefore maintained here by hand and published by the
backend to trailpack-core.

## Source

- Primary: [GW2 wiki — Event timers](https://wiki.guildwars2.com/wiki/Event_timers)
  (content license: CC BY-NC-SA 2.5 — keep attribution in published data).
- **Workaround (since 2026-10-31):** the wiki currently redirects every request
  to an AWS Cognito login and is not publicly reachable. Data is taken from the
  Wayback Machine instead
  (latest known snapshot: `web.archive.org/web/20261026205109/https://wiki.guildwars2.com/wiki/Event_timers`).

## Maintenance rule (for every agent)

Whenever working on event data — and at least at the start of every phase —
check whether `https://wiki.guildwars2.com/wiki/Event_timers` answers with
HTTP 200 without a login redirect. If it does, switch back to the live wiki,
remove the Wayback workaround from this README and from
`docs/plans/backend.md`, and re-verify the data against the live page.

## License

The event data in this directory is licensed under **CC BY-NC-SA 2.5** (see
[`LICENSE`](LICENSE)), because it is derived from the GW2 wiki. The rest of the
repository is MIT. Consequences: attribution required, no commercial use of
the event data, derivatives under the same license. The generated
`events.json` carries the attribution in its metadata.

## Format

One **TOML** file per category (e.g. `world_bosses.toml`, `meta_events.toml`),
hand-maintained; comments are used for sources and change notes.
`trailpack-datagen` validates the files (required fields, valid times, unique
ids) and generates `/v1/events.json` for trailpack-core.

Per event (`[[event]]` table):

| Field | Type | Notes |
|-------|------|-------|
| `id` | string | stable slug, unique across all files, never reused |
| `category` | string | category slug (matches the file) |
| `name` | `{ en, de }` | display names |
| `map_id` | integer | GW2 map id |
| `waypoint` | string | chat link, e.g. `[&BNABAAA=]` |
| `wiki` | `{ en, de }` | wiki page titles for "open wiki page" (event E11) |
| `duration_min` | integer | duration in minutes |
| schedule | `times_utc = ["HH:MM", …]` **or** `start_utc` + `interval_min` | exactly one form |

Example (values illustrative, not verified):

```toml
# Source: wiki Event timers (Wayback snapshot 2026-10-26)
[[event]]
id       = "tequatl"
category = "world_bosses"
name     = { en = "Tequatl the Sunless", de = "Tequatl der Sonnenlose" }
map_id   = 53
waypoint = "[&BNABAAA=]"
wiki     = { en = "Tequatl the Sunless", de = "Tequatl der Sonnenlose" }
duration_min = 15
times_utc = ["00:00", "03:00", "07:00", "11:30", "16:00", "19:00"]
```
