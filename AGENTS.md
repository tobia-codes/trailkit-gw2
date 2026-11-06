# AGENTS.md

Shared rules for Codex and Claude Code. `CLAUDE.md` imports this file;
`.claude/skills` links to `.agents/skills`. Maintain one shared rule set.

## Start and load context

- Read `docs/plans/NEXT_SESSION.md`: **Resume here** and **Recurring checks**.
  Preserve current work and follow due checks. Project: GW2 Nexus addon DLLs
  plus a Rust data pipeline; backend deferred. Plans: `docs/plans/README.md`.
- Load only the task's skill and relevant reference sections below. Search
  with `rg` before reading large files; do not recursively load linked docs
  or reread unchanged context. Do not load both Rust guides in full.
- Run recurring checks when triggered, using
  [the reference](docs/agent-reference.md#recurring-checks). Report blocked
  checks honestly; do not silently count them as completed.

| Task | Read when needed |
|------|------------------|
| Unsettled decision | `.agents/skills/grilling/SKILL.md` |
| Addon/runtime code in `addons/` or `crates/` | `.agents/skills/nexus-addon-dev/SKILL.md`; structure/threading/FFI sections relevant to the task |
| Addon UI | `.agents/skills/trailkit-ui/SKILL.md` and `references/current-design.md`; relevant token/UX rules |
| Rust writing/review | `.agents/skills/rust-best-practices/SKILL.md`; specific chapters only when needed |
| Detailed Rust rule | `.agents/skills/rust-skills/SKILL.md`; look up the specific rule |
| Files, names, build, deploy, checks | [Agent reference](docs/agent-reference.md) — relevant section only |

## Hard rules

1. **Publish only after approval (R74).** Work and commit logical steps on
   `feature/`, `chore/`, `bugfix/` or `hotfix/` branches. New branches start
   from freshly fetched `origin/develop`; keep follow-ups on the active branch.
   For addon changes, deploy with `cargo xtask deploy <addon>` and let the user
   test in game before asking to merge. Only after their explicit OK in the
   **current conversation**: push the work branch, open the PR into `develop`,
   arm `gh pr merge --auto --merge --delete-branch`. Every required CI check
   must pass; fix failures, never bypass. Docs/plans need the same merge OK.
   Group related commits in one PR. Direct pushes to `develop`/`main` are
   forbidden. Every other publication needs separate explicit approval each
   time: `main` merges, release PRs, tags, releases, force pushes, protection
   or CI workflow changes. Exception: the scheduled data-pipeline job may
   publish `gh-pages` and archive releases (D9); changing its workflow still
   needs approval. Historical R69 and tool allowlists grant no standing OK.
2. **Decisions belong to the user.** For any decision not settled in plans or
   by the user (architecture, scope, libraries, naming, UX, data model, etc.),
   use `grilling`: numbered rounds, a recommendation for each, wait for answers.
   Do not silently assume or reopen settled decisions. Record decisions in
   the relevant plan; do not ask the user for facts you can inspect yourself.
   **Visual approval first (R87).** Before every visible addon design change
   (even small layout, wording, typography, color, icon or state changes), show
   a labelled visual proposal and get explicit approval in this conversation.
   Show every affected state in Dark/Light, plus minimum width and German text
   where layout is affected. Isolated prototypes are allowed before approval;
   addon implementation waits. Material deviations need another visual OK.
   Afterwards review the whole changed snapshots against `tokens.md` §10
   (R82), then test in game and obtain merge approval separately.
3. **Every project has a README:** purpose, build and usage for each
   crate/addon/app/package; keep it current when behavior changes.
4. **Document every item:** functions, methods, structs, enums, traits and
   modules, public/private. Use Rust `///`/`//!`, JSDoc/TSDoc or Python
   docstrings; include relevant errors, panics and safety contracts.
5. **English in files.** Code, identifiers, comments, docs, plans and commits
   are English; addon strings are localized en/de, never inline UI literals.
6. **Help stays current.** User-visible behavior changes update
   `addons/<name>/help/{en,de}.md` in the same change.
7. **Share on real reuse.** Search the workspace before adding a helper/type.
   Extract at the second real use, replacing all copies in the same change,
   never speculatively. Shared UI comes from `crates/trailkit-ui`.
8. **Type everything.** Prefer typed/newtype IDs, enums, serde structs and
   checked `sqlx::query!`; untyped JSON belongs only at boundaries.
   Make invalid states unrepresentable.
9. **Performance and security.** Follow Nexus threading/FFI/API-key
   rules; performance beats DLL size (R72). Treat keys and user data as secrets.
10. **GDPR is mandatory.** Every service/provider/log/backup/CDN/site must
   comply: minimize personal data (IPs count), prefer EU providers with
   DPA/AVV, no tracking or
   analytics without consent, and flag privacy impact to the user.

Never bypass a hook, refusal or sandbox through another command/flag; use
supported tool approvals and report blocks. Conventional Commits, never
`--amend`, no AI attribution/footer/trailer. Public commit email:
`tobiacodes@gmail.com`; never a private address.

## Naming conventions

Readable names, standard Rust casing, no opaque abbreviations or catch-all
names. Paths are `kebab-case`, Rust modules `snake_case`, crates `trailkit-*`,
data keys `snake_case`, Nexus IDs typed `trailkit.<addon>.<kind>.<name>` constants.
Before adding/renaming paths or identifiers, consult the binding
[full conventions](docs/agent-reference.md#naming-conventions), including exceptions.

## Validation

- Before cargo: `source ~/.cargo/env`. Rust changes: `cargo fmt --all`,
  `cargo clippy --all-targets -- -D warnings`, `cargo test`.
- Addon/Windows code also needs Windows clippy; UI changes need snapshot tests
  and visual review. Commands: [reference](docs/agent-reference.md#development-environment).
- Changed Rust: `cargo xtask check-file-length` (R77). For hints on touched
  files/functions, propose a split by concern via grilling; never split unasked.
- Paths: `cargo xtask check-names`; tokens: `cargo xtask tokens`; added icons:
  `cargo xtask subset-fonts`. Generated token values are never edited by hand.

## Work and communicate efficiently

German conversation; explain Rust concepts with TypeScript comparisons when
helpful. State evidence, trade-offs and unverified assumptions plainly.
Read narrow ranges, cap/filter tool output, batch independent reads, avoid
reprinting code/plans and use file links/decision IDs. Review only new/changed
snapshots, the whole element. Cover each affected state once; no unnecessary
cross product of independent states (tokens.md §10). Start with a small proposal.
Use sub-agents only for independent work whose benefit justifies extra context;
never for a trivial lookup. Keep progress/final responses concise. For long
sessions, update the handoff and suggest a fresh session.
