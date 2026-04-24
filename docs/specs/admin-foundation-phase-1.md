# Admin Foundation Phase 1 Spec

> Status: proposed baseline for the first product-capability phase of `rustzen-admin`

## Goal

Define the first real product-capability phase for `rustzen-admin` as a reusable admin foundation instead of a generic system shell or an early business platform.

This phase should establish a stable, reusable baseline that later projects can build on directly.

## Positioning

`rustzen-admin` in Phase 1 is a reusable product foundation for admin systems.

It is not:

- a vertical business product
- a multi-tenant platform
- a plugin-driven platform
- a workflow engine
- a "feature-rich" template that mixes unrelated modules

Phase 1 exists to deliver the smallest feature set that makes the repository usable as a real admin foundation.

## Product Direction

The first product-capability phase should favor reusable platform-adjacent admin capabilities over business-domain modules.

The repository should optimize for:

- clear module boundaries
- low structural regret
- direct reuse in downstream projects
- explicit backend and frontend ownership

It should not optimize for:

- breadth of feature count
- speculative extensibility
- compatibility with old structure
- early business-domain abstraction

## Phase 1 Feature Groups

Phase 1 should include exactly five top-level feature groups:

- `identity`
- `access`
- `audit`
- `system`
- `runtime`

These groups form the minimum reusable admin baseline.

## Feature Group Responsibilities

### `identity`

Owns user identity capabilities that answer: who is the current user and how does the current user manage their own account state.

Includes:

- login
- logout
- current user
- personal profile
- change password

Does not include:

- role management
- menu management
- permission assignment
- complex credential recovery flows

### `access`

Owns access-control capabilities that answer: what can the current user see and do.

Includes:

- roles
- permissions
- menus
- role-permission assignment
- role-menu assignment
- visible-menu resolution for the current user
- backend permission enforcement contracts

Does not include:

- login-state ownership
- personal profile ownership
- data-level policy engines
- expression-based authorization systems

### `audit`

Owns reusable action-history and traceability capabilities.

Includes:

- login logs
- operation logs
- key admin action trails

Does not include:

- permission decisions
- business workflow history
- archival/reporting pipelines

### `system`

Owns reusable global admin support data.

Includes:

- dictionaries
- system configuration
- foundational options sources consumed by frontend forms and pages

Does not include:

- access-control ownership
- file-resource ownership
- tenant or organization models
- dynamic schema or metadata engines

### `runtime`

Owns reusable runtime resource capabilities.

Includes:

- file upload
- file metadata
- resource access path conventions
- basic file lifecycle constraints

Does not include:

- notifications
- task orchestration
- storage-provider abstraction
- media-processing pipelines

## Cross-Group Rules

- `identity` answers who the current user is.
- `access` answers what the current user can do.
- `audit` records what happened.
- `system` provides reusable support data.
- `runtime` provides reusable resource handling.

No feature group should absorb another group's primary ownership.

In particular:

- `identity` must not absorb authorization ownership
- `access` must not be hidden under `system`
- `audit` must record actions but not decide policy
- `runtime` must not become a generic bucket for unrelated infrastructure features

## Phase 1 Minimum Deliverables

### `identity`

Must deliver:

- login
- logout
- current-user query
- self profile update
- self password change

Will not deliver in Phase 1:

- password reset
- email verification
- phone verification
- MFA
- third-party login

### `access`

Must deliver:

- user-role relation
- role-permission relation
- role-menu relation
- menu tree for current user
- backend route permission checks

Will not deliver in Phase 1:

- policy builders
- condition-based permissions
- data-scope permissions
- visual permission-rule editors

### `audit`

Must deliver:

- login log
- operation log
- key management action traceability

Will not deliver in Phase 1:

- async audit pipelines
- archive tiers
- audit analytics center

### `system`

Must deliver:

- dictionary management
- system configuration management
- frontend-consumable options endpoints

Will not deliver in Phase 1:

- multi-environment config center
- feature-flag framework
- dynamic schema engine

### `runtime`

Must deliver:

- single-file upload
- unified file access path
- file metadata record
- basic delete constraints

Will not deliver in Phase 1:

- multipart upload
- object-storage abstraction
- multi-backend storage support
- media-processing workflows

## Delivery Priority

Phase 1 should prioritize feature groups in this order:

1. `identity`
2. `access`
3. `audit`
4. `system`
5. `runtime`

This order is intentional:

- `identity` and `access` establish the reusable admin backbone
- `audit` raises the foundation from demo-level to operationally usable
- `system` and `runtime` extend reuse value without driving the architecture

## Explicit Exclusions

Phase 1 must not introduce:

- organization management
- tenant management
- project management
- workflow or approval modules
- order or content modules
- plugin systems
- event buses
- multi-tenant abstractions
- generalized metadata engines

These are downstream or later-phase concerns, not foundation-phase requirements.

## Repository Placement

Phase 1 should stay within the current repository shape.

Do not introduce new top-level apps or new shared crates for this phase.

### Backend

Backend feature groups should be represented as top-level feature folders under:

- `zen-server/src/features/identity/`
- `zen-server/src/features/access/`
- `zen-server/src/features/audit/`
- `zen-server/src/features/system/`
- `zen-server/src/features/runtime/`

Rules:

- `system/` should stop acting as a catch-all home for unrelated admin capabilities
- `identity`, `access`, `audit`, and `runtime` should be first-class feature groups when the work starts
- standard feature structure still applies: `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, `types.rs`

### Frontend

Frontend API modules should mirror backend capability groups:

- `zen-web/src/api/identity/`
- `zen-web/src/api/access/`
- `zen-web/src/api/audit/`
- `zen-web/src/api/system/`
- `zen-web/src/api/runtime/`

Frontend routes should also follow capability boundaries:

- `zen-web/src/routes/profile/`
- `zen-web/src/routes/access/`
- `zen-web/src/routes/audit/`
- `zen-web/src/routes/system/`
- `zen-web/src/routes/runtime/`

### Documentation

Future Phase 1 follow-up docs should use the same boundaries:

- one foundation-level rollout plan
- feature-group-specific specs only when a group needs deeper design work

## Structural Rules

- Do not rename `identity` back to `auth` at the product-capability level.
- Do not hide `access` under `system`.
- Do not let `runtime` become a generic infrastructure bucket.
- Do not add speculative extension surfaces before the first baseline closes.

## Success Criteria

Phase 1 is successful when:

- `rustzen-admin` can act as a real admin starting point for downstream projects
- identity and access are reusable without business-module assumptions
- audit exists as a first-class reusable capability
- system support data and runtime file handling are available as foundation services
- repository structure stays clear and does not absorb speculative platform concerns

## Follow-Up

The next artifact after this spec should be a rollout plan that:

- converts the five feature groups into phased implementation work
- identifies which existing modules move, stay, or split
- defines the first implementation slices for `identity`, `access`, `audit`, `system`, and `runtime`
