//! Trailkit REST API server (ROADMAP R90, `docs/plans/backend.md`).
//!
//! **Skeleton only.** The backend is deferred until a feature needs dynamic
//! queries such as the price history (ROADMAP phase 6, backlog BL5). The
//! crate exists so the repository layout is ready (Q172–Q180); it has no
//! dependencies and does nothing yet. The web stack is decided with the first
//! real endpoint; settled so far are Postgres via `sqlx` (B6) and a Docker
//! image behind Caddy (B10).

/// Entry point. Does nothing until the first endpoint is implemented.
fn main() {}
