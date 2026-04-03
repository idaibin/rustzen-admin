# rustzen-admin

A structured monorepo foundation for Rust full-stack admin systems.

> `rustzen-admin` combines an Axum backend, a React frontend, and repository-level documentation in a single codebase designed for clear boundaries, maintainability, and AI-friendly collaboration.

## Overview

`rustzen-admin` is an open-source full-stack admin system foundation built for real-world projects, not just isolated UI demos.

The repository is organized as a monorepo:

- `server/` contains the Rust backend application
- `web/` contains the React frontend application
- `docs/` contains repository-level architecture and development guides
- the root keeps shared commands, workspace metadata, and collaboration entry documents

This layout keeps backend, frontend, and repository rules explicit, making the codebase easier to understand, review, and evolve.

## Why this repository

Many admin repositories optimize for getting pages running quickly, but become harder to maintain once features, permissions, and data flow start to grow.

`rustzen-admin` is built around a different goal:

- explicit backend and frontend boundaries
- feature-oriented backend organization
- repository-level documentation and collaboration rules
- synchronized changes across code, contracts, and docs
- a structure that is easier for contributors and AI tools to work with

## Repository Layout

```txt
.
├── server/
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
├── web/
│   └── src/
│       ├── routes/
│       ├── api/
│       ├── components/
│       │   └── base-layout/
│       └── stores/
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── pnpm-workspace.yaml
```

## Documentation Entry Points

- [AGENTS.md](./AGENTS.md): repository-level collaboration rules
- [server/AGENTS.md](./server/AGENTS.md): backend entry guide
- [web/AGENTS.md](./web/AGENTS.md): frontend entry guide
- [docs/architecture.md](./docs/architecture.md): repository structure, boundaries, and command entrypoints
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
