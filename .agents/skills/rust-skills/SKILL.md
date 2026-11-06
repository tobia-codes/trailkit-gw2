---
name: rust-skills
description: >
  Comprehensive Rust coding guidelines with 265 rules across 26 categories.
  Use when writing, reviewing, or refactoring Rust code. Covers ownership,
  error handling, async patterns, concurrency, unsafe code, API design, memory
  optimization, performance, numeric safety, conversions, serde, pattern
  matching, macros, closures, observability, testing, and common anti-patterns.
  Invoke with /rust-skills.
license: MIT
metadata:
  author: leonardomso
  version: "1.5.1"
  sources:
    - Rust API Guidelines
    - Rust Performance Book
    - Rust 2024 Edition Guide
    - The Rustonomicon
    - ripgrep, tokio, serde, polars, axum, cargo codebases
---

# Rust rule lookup

Use this skill for a concrete Rust question, not as a default handbook read.
Project rules in AGENTS.md, nexus-addon-dev and trailkit-ui take precedence
over generic recommendations. Workspace lints remain authoritative.

## Lookup workflow

1. Select the relevant prefix below for the current code change or review.
2. Search `references/rule-index.md` or filenames under `rules/` for the
   specific concern. Read only matching rule files or the relevant section.
3. Apply the rules needed for the change; do not load the entire catalogue
   or every category. Reuse guidance already read during this session.
4. Report concrete review findings with code locations and impact; the
   catalogue is not an obligatory checklist for every edit.

The original 265-rule catalogue, Cargo examples and attribution remain in
[the rule index](references/rule-index.md). Individual rule files remain in
`rules/`; names and LICENSE are preserved (local adaptation R88).

## Rule Categories by Priority

| Priority | Category | Impact | Prefix | Rules |
|----------|----------|--------|--------|-------|
| 1 | Ownership & Borrowing | CRITICAL | `own-` | 12 |
| 2 | Error Handling | CRITICAL | `err-` | 12 |
| 3 | Memory Optimization | CRITICAL | `mem-` | 17 |
| 4 | Unsafe Code | CRITICAL | `unsafe-` | 7 |
| 5 | API Design | HIGH | `api-` | 17 |
| 6 | Async/Await | HIGH | `async-` | 18 |
| 7 | Concurrency | HIGH | `conc-` | 4 |
| 8 | Compiler Optimization | HIGH | `opt-` | 12 |
| 9 | Numeric & Arithmetic Safety | HIGH | `num-` | 5 |
| 10 | Type Safety | MEDIUM | `type-` | 13 |
| 11 | Trait & Generics Design | MEDIUM | `trait-` | 6 |
| 12 | Conversions | MEDIUM | `conv-` | 3 |
| 13 | Const & Compile-Time | MEDIUM | `const-` | 4 |
| 14 | Serde | MEDIUM | `serde-` | 8 |
| 15 | Pattern Matching | MEDIUM | `pat-` | 5 |
| 16 | Macros | MEDIUM | `macro-` | 8 |
| 17 | Closures | MEDIUM | `closure-` | 5 |
| 18 | Collections | MEDIUM | `coll-` | 4 |
| 19 | Naming Conventions | MEDIUM | `name-` | 16 |
| 20 | Testing | MEDIUM | `test-` | 15 |
| 21 | Documentation | MEDIUM | `doc-` | 12 |
| 22 | Observability | MEDIUM | `obs-` | 7 |
| 23 | Performance Patterns | MEDIUM | `perf-` | 13 |
| 24 | Project Structure | LOW | `proj-` | 14 |
| 25 | Clippy & Linting | LOW | `lint-` | 13 |
| 26 | Anti-patterns | REFERENCE | `anti-` | 15 |

---
