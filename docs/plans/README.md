# Plans

| Document | Purpose |
|----------|---------|
| [`ROADMAP.md`](ROADMAP.md) | Master plan: order of implementation, shared foundations |
| [`addons/`](addons/) | One plan per Nexus addon: [memo](addons/trailkit-memo.md), [core](addons/trailkit-core.md), [event](addons/trailkit-event.md) (new ones copy [`plan-template.md`](plan-template.md)) |
| [`data-pipeline.md`](data-pipeline.md) | **Current:** static data files + daily price archive (no server) |
| [`backend.md`](backend.md) | Rust server — deferred target design |
| [`BACKLOG.md`](BACKLOG.md) | Postponed ideas |
| [`NEXT_SESSION.md`](NEXT_SESSION.md) | Maintainer handoff: where the work stands and what comes next |

Plans record **decisions**, each with an id, a date and who decided:
`R…` roadmap, `M…` memo, `B…` backend, `D…` data pipeline, `E…` event addon,
`BL…` backlog, `Q…` the question that settled it. Code and docs cite these
ids (e.g. "R76") so the reason for a rule can always be looked up.
Decisions are taken by the maintainer in numbered question rounds (the
`grilling` skill); open questions stay in the plan's *Open questions* section
until settled. Contributors propose changes in an issue, not by editing plans.
