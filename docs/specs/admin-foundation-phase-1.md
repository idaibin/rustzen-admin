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

Phase 1 should include exactly six practical starter groups:

- `auth`
- `account`
- `rbac`
- `audit`
- `system`
- `runtime`

These groups form the minimum reusable admin baseline.

## Feature Group Responsibilities

### `auth`

Owns authentication and current-session bootstrap.

Includes:

- login
- logout
- current user

Does not include:

- personal profile ownership
- role management
- menu management
- permission assignment
- complex credential recovery flows
- OAuth2, OIDC, SAML, SSO, MFA, or enterprise directory features

### `account`

Owns the current administrator's self-service account state.

Includes:

- personal profile
- avatar update
- change password

Does not include:

- role management
- menu management
- permission assignment
- administrator-driven user lifecycle actions

### `rbac`

Owns starter-level role-based access-control capabilities.

Includes:

- roles
- permissions
- menus
- role-permission assignment
- role-menu assignment
- visible-menu resolution for the current user
- backend permission enforcement contracts
- administrative user management needed for assigning roles

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

- `auth` answers whether the current user is authenticated.
- `account` answers how the current user manages their own account.
- `rbac` answers what the current user can see and do.
- `audit` records what happened.
- `system` provides reusable support data.
- `runtime` provides reusable resource handling.

No feature group should absorb another group's primary ownership.

In particular:

- `auth` must not absorb account, RBAC, or IAM-platform ownership
- `account` must not absorb administrator user management
- `rbac` must not be hidden under `system`
- `audit` must record actions but not decide policy
- `runtime` must not become a generic bucket for unrelated infrastructure features

## Phase 1 Minimum Deliverables

### `auth`

Must deliver:

- login
- logout
- current-user query

Will not deliver in Phase 1:

- password reset
- email verification
- phone verification
- MFA
- third-party login
- OAuth2 or OIDC provider behavior

### `account`

Must deliver:

- self profile update
- self avatar update
- self password change

Will not deliver in Phase 1:

- account recovery
- device management
- session management UI

### `rbac`

Must deliver:

- admin user management
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

1. `auth`
2. `account`
3. `rbac`
4. `audit`
5. `system`
6. `runtime`

This order is intentional:

- `auth`, `account`, and `rbac` establish the reusable admin backbone
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

Current Phase 1 backend ownership uses existing folders where they already work:

- `zen-server/src/features/auth/`
- `zen-server/src/features/account/`
- RBAC is currently carried by `zen-server/src/features/system/role/`, `zen-server/src/features/system/menu/`, and access-facing parts of `zen-server/src/features/system/user/`
- audit is currently carried by `zen-server/src/features/system/log/`
- system support data is currently carried by `zen-server/src/features/system/dict/`
- runtime file handling is currently carried by `zen-server/src/common/files.rs` and runtime wiring in `zen-server/src/infra/`

Rules:

- do not create new feature folders just to match capability labels
- standard feature structure still applies when a new backend feature is actually needed: `mod.rs`, `handler.rs`, `service.rs`, `repo.rs`, `types.rs`

### Frontend

Current Phase 1 frontend ownership uses existing routes and API modules where they already work:

- `zen-web/src/api/auth/`
- `zen-web/src/api/account/`
- `zen-web/src/api/system/`
- `systemAPI.role`, `systemAPI.menu`, and `systemAPI.user` are the current RBAC API carriers
- `systemAPI.log` is the current audit API carrier
- `systemAPI.dict` is the current system support-data API carrier

Current routes:

- `zen-web/src/routes/profile.tsx`
- `zen-web/src/routes/system/`
- `system/role`, `system/menu`, and `system/user` are the current RBAC route carriers
- `system/log` is the current audit route carrier
- `system/dict` is the current system route carrier

### Documentation

Future Phase 1 follow-up docs should use these boundaries:

- one foundation-level rollout plan
- feature-group-specific specs only when a group needs deeper design work

## Structural Rules

- Do not introduce a general IAM product-capability group in Phase 1.
- Do not create `rbac` paths just for naming consistency.
- Do not let `runtime` become a generic infrastructure bucket.
- Do not add speculative extension surfaces before the first baseline closes.

## Success Criteria

Phase 1 is successful when:

- `rustzen-admin` can act as a real admin starting point for downstream projects
- auth, account, and RBAC are reusable without business-module assumptions
- audit exists as a first-class reusable capability
- system support data and runtime file handling are available as foundation services
- repository structure stays clear and does not absorb speculative platform concerns

## Follow-Up

The rollout plan should:

- convert the six starter groups into phased implementation work
- identify which existing modules already satisfy the baseline
- defines the first implementation slices for `auth`, `account`, `rbac`, `audit`, `system`, and `runtime`
