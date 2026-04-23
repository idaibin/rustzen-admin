# Repository Comparison

> Baseline comparison of `rustzen-admin`, `rustzen-inspect`, and `zen-clear`, written for future updates, refactoring, and optimization decisions.

## Scope

- Date: `2026-04-22`
- Current repository: `rustzen-admin`
- Reference repositories:
  - `/Users/daibin/Projects/repo-github/rustzen-inspect`
  - `/Users/daibin/Projects/repo-github/zen-clear`

## Snapshot

| Repository | Product role | Main runtime shape | Current structure signal |
| --- | --- | --- | --- |
| `rustzen-admin` | full-stack admin foundation | `zen-core` + `zen-server` + `zen-web` | clear base repo, limited domain breadth, documentation-first restructuring |
| `rustzen-inspect` | self-hosted monitoring platform | `zen-core` + `zen-common` + `zen-server` + `zen-agent` + `zen-web` | strongest backend and deployment maturity, clear multi-runtime split |
| `zen-clear` | macOS developer disk cleanup product | `zen-core` + `zen-cli` + `zen-gui` + `zen-cloud` + `zen-website` | strongest product-surface split, strongest domain-engine extraction |

## Repository Profiles

### `rustzen-admin`

- Positioning is explicit: it is a monorepo foundation for Rust admin systems, not a finished vertical product.
- The core split is clean: shared auth and permission code in `zen-core/`, business backend in `zen-server/`, frontend in `zen-web/`.
- Backend shape is already usable as a template:
  - top-level features: `auth/`, `dashboard/`, `system/`
  - `system/` is further split into `dict/`, `log/`, `menu/`, `role/`, `user/`
- Frontend route coverage matches the current admin baseline: login, dashboard-like index, and system management pages.
- Documentation discipline is good and already close to a reusable starter repo.
- The main limitation is not architecture quality, but product depth. Compared with the other two repositories, the current repo is still a smaller base layer.

### `rustzen-inspect`

- This repo is already a real product, not just a scaffold.
- The key architectural step is the dual runtime split:
  - `zen-server/` is the control plane
  - `zen-agent/` is the node-side executor and collector
  - `zen-common/` carries shared runtime code that does not belong in auth/permission
- Backend feature coverage is much richer than `rustzen-admin`:
  - server features include `agent`, `alert`, `bootstrap`, `cors`, `deploy`, `node`, `project`, `reports`, `system`, `user`
  - agent features include `alerts`, `config`, `deploy`, `health`, `metrics`
- Deployment and runtime contracts are more mature:
  - separate server and agent binaries
  - explicit YAML config rules
  - explicit runtime directory layout
  - versioned build outputs in `target/versions/`
- The documentation set is also more operational. It not only explains code layout, but also explains deployment, runtime paths, build artifacts, and cross-node rules.
- This repo is the closest reference for how `rustzen-admin` should evolve if it grows from a base repo into a deployable platform product.

### `zen-clear`

- This repo is not organized around server features first. It is organized around product surfaces and a reusable domain engine.
- The central idea is strong separation between domain logic and delivery surfaces:
  - `zen-core/` owns scan, analysis, cleanup, restore, rules, safety, and typed models
  - `zen-cli/` is a thin command interface
  - `zen-gui/` is a Tauri desktop app
  - `zen-cloud/` is a Next.js cloud control plane
  - `zen-website/` is a public marketing/documentation site
- This repo shows a different maturity pattern from `rustzen-inspect`:
  - `rustzen-inspect` is platform-oriented
  - `zen-clear` is product-surface-oriented
- Its documentation model is also stricter than the current repo. It has separate layers for strategy, specs, runtime execution state, evidence, and archive.
- This repo is the best reference when `rustzen-admin` needs stronger product packaging, clearer domain-engine extraction, or more explicit documentation lifecycle management.

## Shared Patterns Across The Three Repositories

- All three repositories treat the root as a coordination layer instead of a business layer.
- All three repositories push Rust domain logic into dedicated crates instead of mixing everything into one app.
- All three repositories use docs and `AGENTS.md` as part of the engineering contract, not as optional notes.
- All three repositories prefer explicit structure over compatibility code and fallback paths.
- All three repositories use `justfile` or equivalent top-level commands as the main developer entrypoint.

## Core Differences

| Topic | `rustzen-admin` | `rustzen-inspect` | `zen-clear` |
| --- | --- | --- | --- |
| Main goal | reusable admin baseline | monitoring platform | disk cleanup product |
| Rust shared layer | `zen-core` only | `zen-core` + `zen-common` | `zen-core` as domain engine |
| Backend runtime count | single backend | server + agent | local core + desktop + cloud |
| Frontend surfaces | one admin web | one admin web | desktop GUI + cloud app + website |
| Domain depth | light | heavy | heavy |
| Deployment complexity | medium | high | high |
| Documentation maturity | good baseline | strong operational spec | strong lifecycle and product spec |

## What `rustzen-admin` Already Does Well

- The repository boundary is simpler and clearer than the other two.
- Shared auth and permission capability is isolated early, which is the right long-term decision.
- The backend feature pattern is stable and understandable.
- The frontend and backend are still close enough that contract synchronization is manageable.
- The current structure is small enough to remain a clean starter, which is a real advantage and should be preserved.

## Gaps In `rustzen-admin` Relative To The Other Two

- It lacks a second shared Rust layer like `zen-common/`, so any future cross-feature runtime utilities still tend to accumulate inside the backend.
- Its deployment story is lighter than `rustzen-inspect`; build artifacts, runtime layout, and operational contracts are less productized.
- Its domain coverage is still thin; `dashboard` plus `system/*` is enough for a base repo, but not enough for a stronger opinionated admin product.
- Its document set is clean, but still less decision-oriented than `zen-clear` and less operations-oriented than `rustzen-inspect`.
- It does not yet have a clear “growth path” document that explains what should happen if this repo stays a starter versus becomes a product.

## Direct Lessons For `rustzen-admin`

### Keep

- Keep the current monorepo shape.
- Keep `zen-core/` focused on shared auth and permission.
- Keep the current feature-first backend structure.
- Keep docs single-responsibility and synchronized with code changes.

### Borrow From `rustzen-inspect`

- Add a second shared crate only when truly needed for cross-feature or cross-runtime logic.
- Make runtime and deployment contracts more explicit as soon as the repo starts shipping outside local development.
- Keep feature directories fully self-contained and document any intentional deviations from the standard five-file pattern.
- Use clearer artifact naming and release output conventions if packaging becomes a standard workflow.

### Borrow From `zen-clear`

- If the domain grows, extract reusable domain logic before adding more delivery surfaces.
- Keep app surfaces thin and let the core domain layer own the important rules.
- Strengthen document lifecycle only when the number of active plans/specs actually requires it.

### Do Not Copy Yet

- Do not add `zen-common`-style layering before there is shared logic that clearly does not belong in `zen-core/`.
- Do not copy `rustzen-inspect` server-agent complexity unless a second runtime actually exists.
- Do not copy `zen-clear` cloud, website, or desktop surface splitting unless the product scope requires it.
- Do not overbuild the docs system into a process-heavy structure while the repo is still primarily a foundation project.

## Suggested Optimization Direction

1. Keep `rustzen-admin` positioned as the clean baseline repo, not as a reduced copy of `rustzen-inspect`.
2. Define the next layer of backend features that make the starter more opinionated without turning it into a specific product.
3. Introduce a second shared Rust crate only after shared non-auth runtime code appears in at least two places.
4. Tighten release and deployment conventions when packaging becomes a regular workflow, not before.
5. Add a follow-up document later for “starter repo evolution rules” if this repository begins serving multiple downstream products.

## Maintenance Rules For This Document

- Update this file when any of the three repositories changes its workspace members, app surfaces, or primary runtime shape.
- Update this file when `rustzen-admin` adds a new shared crate, a new delivery surface, or a major backend feature group.
- Keep this document comparative and structural.
- Do not turn this file into a task list, execution log, or changelog.
