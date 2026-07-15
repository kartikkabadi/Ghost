# Ghost OSINT Harness

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](./LICENSE)

Ghost is an autonomous OSINT research system built around a provider gateway, a reasoning/orchestration engine, a SQLite-backed evidence graph, and a reporting layer designed for fast human review.

The project is public and open for contributions. Please read [ETHICAL_USE.md](./ETHICAL_USE.md) and [SECURITY.md](./SECURITY.md) before using or contributing.

## What Ghost does

- Breaks a research objective into evidence-bearing sub-questions.
- Routes sub-questions to the cheapest sufficient provider (LLM, search, crawl, enrichment).
- Gathers candidate sources, normalizes claims, and cross-checks contradictions.
- Scores confidence and surfaces blind spots.
- Emits a structured, analyst-ready brief with sources, caveats, and next actions.

## Architecture

| Layer | Location | Responsibility |
|---|---|---|
| Web UI | `apps/web` | Next.js analyst dashboard and review surface. |
| Contracts | `packages/contracts` | Shared TypeScript types for claims, sources, and reports. |
| Core | `crates/ghost-core` | Rust orchestration, provider gateway, evidence store, and reporting engine. |
| Skills | `skills/` | Capability packages for coordinator, worker, verifier, and reporter agents. |
| Docs | `docs/architecture.md` | System shape and operating rules. |

## Current subsystems

- **Provider gateway** (`crates/ghost-core/src/gateway/mod.rs`, `provider/mod.rs`)
- **Autonomous research loop** (`crates/ghost-core/src/orchestrator/mod.rs`, `search.rs`)
- **Evidence store and provenance graph** (`crates/ghost-core/src/db/mod.rs`)
- **Reporting engine** (`crates/ghost-core/src/reporting/mod.rs`)
- **Analyst review UI** (`apps/web`)
- **Skill registry** (`skills/`)

## Quick start

### 1. Clone and enter the workspace

```bash
git clone https://github.com/kartikkabadi/Ghost.git
cd Ghost
```

### 2. Configure environment variables

```bash
cp .env.example .env
# Edit .env and add your API keys.
```

Ghost reads provider credentials from environment variables. No keys are hardcoded in the repository.

### 3. Run the Rust core checks

```bash
cd crates/ghost-core
cargo check
```

### 4. Run the Next.js web app

```bash
cd apps/web
npm install
npm run dev
```

The app will start on [http://localhost:3000](http://localhost:3000).

## Supported providers

Ghost can route work through any provider that implements the provider interface. The current defaults use:

- **OpenRouter** for model routing (`OPENROUTER_API_KEY`)
- **xAI / Grok** for reasoning and scheduler tasks (`XAI_API_KEY`)
- **Firecrawl** for extraction (`FIRECRAWL_API_KEY`)
- **Exa** for search (`EXA_API_KEY`)
- **Reddit** public API for trend signals (`REDDIT_TRENDS_URL`, optional)

See `config.yaml` for default base URLs and model profiles.

## Development

```bash
# Rust
cd crates/ghost-core
cargo check
cargo test

# TypeScript / Next.js
cd apps/web
npm install
npm run build
npm run lint
```

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md).

## Ethical use and security

- [ETHICAL_USE.md](./ETHICAL_USE.md) — acceptable and prohibited use cases.
- [SECURITY.md](./SECURITY.md) — how to report vulnerabilities and handle credentials.

## License

[MIT License](./LICENSE)
