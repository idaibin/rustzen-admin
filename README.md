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
- `docs/` contains repository-level architecture and development guides
- the root keeps shared commands, workspace metadata, and collaboration entry documents

This layout keeps backend, frontend, and repository rules explicit, making the codebase easier to understand, review, and evolve.

## Repository Layout

→ Architecture summary: [docs/architecture.md](./docs/architecture.md)

## Documentation

→ Complete documentation index: [docs/README.md](./docs/README.md)

## Command Source

Use the root `justfile` as the command source of truth; inspect the relevant target before running it.

## Demo

- Local demo URL: `[https://rustzen-admin.idaibin.dev](https://rustzen-admin.idaibin.dev)`
- Demo username: `superadmin`
- Demo password: `rustzen@123`

## Notes

- `README.md` and `AGENTS.md` stay as lightweight entry documents.
- `docs/history/` contains historical execution records and is not current implementation truth.
