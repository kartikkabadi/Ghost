# Ghost OSINT Harness

Ghost is an autonomous OSINT research system built around a provider gateway, a reasoning/orchestration engine, a SQLite-backed evidence graph, and a reporting layer designed for fast human review.

## Architecture goals
- Web UI only: no terminal-first surface area
- Fast Rust core with TypeScript product surface
- SQLite as the default local persistence layer
- Provider routing across LLM, search, crawl, and enrichment vendors
- Multi-source verification before claims are promoted into reports
- Constraint-aware deep reasoning loops that explicitly report blind spots

## Current scaffold
- `crates/ghost-core`: Rust orchestration and service core
- `apps/web`: Next.js + shadcn-style UI shell
- `packages/contracts`: shared TypeScript contracts and schemas
- `docs/architecture.md`: system shape and operating rules

## Planned subsystems
1. Provider gateway
2. Autonomous research loop
3. Evidence store and provenance graph
4. Reporting engine
5. Analyst review UI
6. Evaluation and regression harness
