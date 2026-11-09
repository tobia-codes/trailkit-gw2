# bruno-gw2-api

## Purpose

A [Bruno](https://www.usebruno.com/) collection for the Guild Wars 2 API v2,
so developers can try every active endpoint by hand while building addons and
the data pipeline.

## Status

Developer tool, not shipped with any addon. Covers every active route listed
at <https://api.guildwars2.com/v2> on 2026-11-09.

## Contents

| Path | Content |
|------|---------|
| `bruno.json`, `collection.bru` | Collection settings: header `X-Schema-Version: latest`, bearer auth with `{{api_key}}` |
| `environments/gw2-live.bru` | Variables (`base_url`, `lang`, `character_name`, `guild_id`, `world_id`, `wvw_match_id`, `pvp_season_id`) and the secret `api_key` |
| `public/` | Endpoints without a key (`auth: none`), grouped by first path segment |
| `account/` | Endpoints that need a key (`auth: inherit`): account, characters, guild, commerce transactions, PvP stats, tokens |

Each request lists its optional query parameters disabled (`ids`, `page`,
`page_size`, …) and links its wiki page in the Docs tab. Routes the API marks
as disabled (for example `account/mail`, `achievements/daily`, `events`,
`vendors`) and pure index routes (`home`, `homestead`, `mounts`, `pvp`) are
left out. `commerce/exchange` and `commerce/transactions` are split into their
real sub-routes.

## Usage

1. Open the folder in Bruno (*Open Collection*).
2. Select the environment `gw2-live`.
3. Create a key at <https://account.arena.net/applications> and enter it as
   `api_key` in the environment. It is a secret variable: Bruno keeps its
   value locally and never writes it into the `.bru` files.
4. Set `character_name` (exact name; spaces as `%20`) and `guild_id` for the
   character and guild requests.

## Rules

- Never commit an API key; check `git diff` before committing changes to the
  environment. `.env` files are ignored.
- Variable names are `snake_case`, file names `kebab-case`
  ([naming conventions](../../docs/agent-reference.md#naming-conventions)).
- General rules: [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## Links

- Route list: <https://api.guildwars2.com/v2>
- API reference: <https://wiki.guildwars2.com/wiki/API:Main>
- Bruno file format: <https://docs.usebruno.com/bru-lang/overview>
