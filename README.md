# rustzen-admin

A structured monorepo foundation for Rust full-stack admin systems.

> `rustzen-admin` combines an Axum backend, a React frontend, and repository-level documentation in a single codebase designed for clear boundaries, maintainability, and AI-friendly collaboration.

## Overview

`rustzen-admin` is an open-source full-stack admin system foundation built for real-world projects, not just isolated UI demos.

The repository is organized as a monorepo:

- `zen-core/` contains shared auth and permission capabilities for Rust services
- `zen-server/` contains the Rust backend application
- `zen-web/` contains the React frontend application
- `deploy/` contains deployment assets and release support files
- `docs/` contains repository-level guides plus goals, plans, specs, and agent-facing documents
- the root keeps shared commands, workspace metadata, and collaboration entry documents

This layout keeps backend, frontend, and repository rules explicit, making the codebase easier to understand, review, and evolve.

## Why this repository

Many admin repositories optimize for getting pages running quickly, but become harder to maintain once features, permissions, and data flow start to grow.

`rustzen-admin` is built around a different goal:

- explicit backend and frontend boundaries
- reusable auth and permission capability layers
- feature-oriented backend organization
- repository-level documentation and collaboration rules
- synchronized changes across code, contracts, and docs
- a structure that is easier for contributors and AI tools to work with

## Repository Layout

```txt
.
├── zen-core/
│   ├── Cargo.toml
│   └── src/
│       ├── auth/
│       ├── permission/
│       ├── error.rs
│       └── lib.rs
├── zen-server/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
│       ├── features/
│       │   ├── auth/
│       │   ├── dashboard/
│       │   └── system/
│       ├── infra/
│       ├── common/
│       └── middleware/
├── zen-web/
│   └── src/
│       ├── routes/
│       ├── api/
│       ├── components/
│       │   └── base-layout/
│       └── store/
├── deploy/
│   ├── sql/
│   │   └── repair_menu_schema.sql
│   ├── binary.Dockerfile
│   ├── release.Dockerfile
│   ├── runtime.Dockerfile
│   └── rustzen-admin.service
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── README.md
```

## Documentation Entry Points

- [CHANGELOG.md](./CHANGELOG.md): release notes and breaking changes (start here when upgrading)
- [docs/README.md](./docs/README.md): documentation system entrypoint and placement rules
- [AGENTS.md](./AGENTS.md): repository-level collaboration rules
- [zen-server/AGENTS.md](./zen-server/AGENTS.md): backend entry guide
- [zen-web/AGENTS.md](./zen-web/AGENTS.md): frontend entry guide
- [docs/architecture.md](./docs/architecture.md): repository structure, boundaries, and command entrypoints
- [docs/goals/product-direction.md](./docs/goals/product-direction.md): product direction and repository intent
- [docs/goals/repository-evolution.md](./docs/goals/repository-evolution.md): near-term repository evolution goals
- [docs/plans/2026-04-22-documentation-governance-rollout.md](./docs/plans/2026-04-22-documentation-governance-rollout.md): rollout plan for documentation governance
- [docs/specs/2026-04-22-documentation-governance.md](./docs/specs/2026-04-22-documentation-governance.md): formal documentation governance spec
- [docs/agents/operating-rules.md](./docs/agents/operating-rules.md): stable agent operating rules
- [docs/agents/current-iteration.md](./docs/agents/current-iteration.md): current documentation iteration state
- [docs/repository-comparison.md](./docs/repository-comparison.md): cross-repository baseline for future updates and optimization
- [docs/project-map.md](./docs/project-map.md): entrypoints and high-frequency change paths
- [docs/backend-guide.md](./docs/backend-guide.md): backend layering, naming, database, and error rules
- [docs/frontend-guide.md](./docs/frontend-guide.md): frontend routing, request, state, and UI rules
- [docs/deployment-guide.md](./docs/deployment-guide.md): deployment and runtime configuration rules
- [docs/permission-guide.md](./docs/permission-guide.md): permission model and usage rules

## Common Commands

```bash
just dev-server
just dev-web
just check
just build
just build-binary
just build-release
just build-image
```

## Project Principles

- clear repository boundaries
- minimal root-level responsibility
- single-purpose documentation
- maintainability over patchwork changes
- explicit architecture conventions
- AI-friendly engineering structure

## Status

The repository is under active restructuring and refinement.

Current focus:

- stabilizing the monorepo layout
- aligning backend, frontend, and docs
- refining repository-level conventions
- building a stronger long-term foundation for feature growth
