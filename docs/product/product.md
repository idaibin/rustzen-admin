# Product Foundation

Status: current foundation specification.

This document is the single authority for current product positioning,
direction, module purposes, boundaries, and non-goals. Source remains
authoritative for delivered behavior; `docs/architecture.md` remains
authoritative for runtime and repository structure.

## Product boundary

RustZen Admin is primarily a lightweight, self-hosted operations and
administration product for developer-operators and small technical teams. It is
secondarily a structured Rust full-stack reference implementation; product
journeys take priority over generic framework demonstrations.

The product is one operations console delivered as one signed release. The
release contains four independently restarted server processes but keeps one
version and one rollback boundary:

- Admin owns identity, RBAC, module control, release management, and the Web
  application.
- Monitoring owns managed nodes, metrics, probes, and incidents.
- Analytics owns product-event collection and analysis.
- Reports owns target-backed browser filling templates and runs.

The current product does not contain a fifth service, a dynamic plugin system,
or independently versioned modules.

## Target users, problems, and principles

The primary-user definition remains an adoption assumption to validate. The
current product addresses five concrete problems:

- administer accounts, roles, permissions, and fixed modules from one place;
- understand installation and managed-service health;
- inspect lightweight product-activity summaries and details;
- execute repeatable browser-based reporting with visible failure evidence;
- install and update the complete product with one recovery boundary.

Product decisions follow these principles:

1. Complete setup-to-result, failure, and recovery loops before adding breadth.
2. Keep self-hosting understandable and configuration overhead bounded.
3. Preserve one product experience while isolating module failures and data.
4. Require explicit authorization, visible status, audit evidence, and
   recoverable outcomes for privileged work.
5. Prefer named user problems over template completeness or speculative
   abstractions.
6. Deepen the four fixed product areas before considering extensibility.

## Users and core journeys

- Owner configures the installation, roles, modules, and releases.
- Admin performs day-to-day operations without owner-only release authority.
- Viewer reads the concrete module capabilities granted by the module
  Manifests.
- An operator can inspect module health, monitor nodes and checks, query
  analytics observations, and execute or inspect report-filling runs without
  leaving the Admin console.

Module failure must not prevent Admin login or make an unrelated module
unavailable.

The retained end-to-end journeys are:

- Admin: sign in, inspect system state, manage users and roles, control modules,
  inspect operations, and manage releases within owner/admin/viewer boundaries.
- Monitoring: inspect health, nodes, recent metrics, checks, and incident state,
  with missing data distinguishable from healthy empty data.
- Analytics: inspect installation-wide activity and raw details without a query
  or collection failure blocking other product areas.
- Reports: define a target-backed template, validate run input, execute browser
  steps, and retain enough live or captured evidence to diagnose failure.

Reports currently retains the complete run input with the execution record and
returns it from run queries. Users must not submit passwords, tokens, or other
secrets until a separate protected-storage and response-redaction boundary is
specified, implemented, and verified. Any workflow that accepts secrets must
deliver that boundary before collecting them.

Across those journeys, loading, empty, validation, permission, business-error,
retry, audit, interruption, and recovery results must remain explicit.

## Product language

| Product term | Current meaning | Product owner |
| --- | --- | --- |
| Admin | Control plane, identity, RBAC, release, and Web host. | Admin |
| Monitoring | Node, metric, probe, and incident operations. | Monitoring |
| Analytics | Instance-wide event overview and detail queries. | Analytics |
| Reports | Target systems, browser filling templates, runs, artifacts, and live frames. | Reports |
| Automation | An internal Reports capability, not a separately shipped module. | Reports |
| Report Center | A possible future cross-module report catalog. It is not implemented. | Deferred |

Technical ownership and stable internal names are defined in
[`architecture.md`](../architecture.md) and
[`project-map.md`](../project-map.md).

## Module purposes and direction

| Product area | Internal name | Current purpose | Direction | Explicit non-goal |
| --- | --- | --- | --- | --- |
| Admin | Admin | Trusted control plane for identity, RBAC, module state, system status, operations, and releases. | Clarify installation, access, diagnosis, update, and recovery. | ERP, generic CRUD generation, low-code admin, or workflow builder. |
| Monitoring | Monitor | Managed-node, metric, probe, and incident operations. | Improve the path from signal to actionable incident for small installations. | Full APM, tracing, log warehouse, or cloud orchestrator. |
| Analytics | Insights | Lightweight installation-wide activity collection, overview, detail, and retention. | Make the retained signals useful before adding event families or segmentation. | Marketing automation, general BI, warehouse, or multi-tenant analytics. |
| Reports | Reports | Controlled browser-filling targets, templates, runs, steps, live frames, artifacts, cancellation, and recovery. | Strengthen authoring, validation, credential boundaries, visibility, and recovery. | Unrestricted scripts, general RPA, document editor, or open-ended browser agent. |

## Confirmed decisions

1. Current modules keep independent failure and data-ownership boundaries.
2. A new module starts from a live comparison with the relevant former
   standalone repository. The comparison preserves product behavior, not old
   directory layout or duplicated platform code.
3. Existing Admin identity, RBAC, deployment, navigation, and Web shell are
   reused. A module must not import a second copy from a former repository.
4. Cross-module code moves to a shared owner only after it has compatible real
   consumers and an independently verifiable contract. Security and protocol
   consistency may justify extraction earlier than ordinary presentation code.
5. Module business meanings, status lifecycles, calculations, data selection,
   and failure semantics remain with the module.
6. Root `DESIGN.md` is the sole current authority for shared visual semantics,
   component semantics, and design-approval records. A new or changed product
   surface still requires a scoped UI specification before frontend
   implementation; `docs/ui/features/*` may retain slice-local behavior and
   acceptance evidence. Structured `docs/ui/` packages such as
   `evaluation.yaml` and `artifact-manifest.yaml` are historical or task-local
   evidence only and do not govern current approval.
7. Dashboard is a control-plane landing page: it shows account totals and
   module health. Detailed host resources remain owned by System Status, while
   module metrics and trends remain on their Monitoring and Analytics pages.
8. Dictionary management is not a current product capability because no
   in-repository workflow consumes it. Its HTTP surface, page, navigation, and
   permission are removed; the historical SQLite table remains dormant for
   upgrade compatibility and is not a supported integration contract.

## Legacy-product decisions

The evidence and path-level comparison are recorded in
[`legacy-module-comparison.md`](../reference/legacy-module-comparison.md).

### Monitoring

Retain and extend the current Monitor implementation. The former
`rustzen-inspect` is a behavior reference for alert policies, agent-side
collection, time-drift handling, and period reports. Do not copy its Admin,
system, project, deployment, permission, or runtime-layout layers.

Alert policy management and period reports are separate future feature slices;
they are not authorized by this foundation specification alone.

### Analytics

Retain the current single-project, instance-wide Analytics behavior. The former
`rustzen-analytics` is the reference for a possible multi-project evolution:
project lifecycle, stable project keys, browser-origin and application-package
allowlists, bounded ingestion, aggregation, and richer project queries.

Multi-project behavior changes event identity, permissions, navigation, and
data ownership. It requires one dedicated feature specification and migration
plan before implementation. The old repository's duplicate Admin and deploy
features must not return.

### Reports and Automation

Retain the current target, flow, run, step, screenshot, artifact, cancellation,
and live-frame loop. The former `rustzen-report` is the behavior reference for
possible account credentials, datasets, upload processing, a richer expression
and group DSL, suspend/resume semantics, scheduled runs, and detailed live job
events.

Each capability is a separate feature slice. The old authentication, users,
system settings, deployment, and application shell are rejected because Admin
already owns them. The browser runtime stays Reports-owned until another real
module needs the same semantics; it must not become a generic workflow engine
in advance.

### Report Center

A cross-module report catalog, common report summary, document renderer, and
artifact catalog are deferred. They require at least two real report producers,
an explicit permission model, and a product specification for degraded module
behavior. They do not justify a fifth process or a new contract crate today.

## Non-goals

- Whole-repository source copying from a former product.
- Compatibility wrappers for former HTTP paths, database names, binaries, or
  deployment layouts.
- A universal business status enum, CRUD framework, form DSL, dashboard
  builder, repository layer, or workflow engine.
- A generic dictionary administration surface without a current product
  consumer.
- Cross-database joins or one module reading another module's SQLite database.
- Moving a capability to a shared package before current consumers prove the
  same contract.

## Decision status

- Confirmed: the current product has Admin, Monitoring, Analytics, and Reports
  in one release; Automation remains part of Reports.
- Confirmed: former products are capability evidence, not whole-product
  migration targets.
- Confirmed: Dashboard is limited to control-plane summary and module health;
  detailed resource and product telemetry stay on their owning pages.
- Confirmed: Dictionary management is removed from the current surface while
  its historical SQLite table is retained as dormant upgrade data.
- Confirmed: the primary positioning is a lightweight self-hosted operations
  product; the reference-implementation role is secondary.
- Confirmed: SQLite and one coherent signed release are the current supported
  operating boundary.
- Assumption: the primary adopter is a developer-operator or small technical
  team managing one installation.
- Assumption: the named former repositories remain the best product-behavior
  references. Revalidate their live default branches before each slice.
- Open: which retained journey currently causes the most user friction and
  should receive the next bounded feature specification.
- Open: which deferred capability becomes the first implementation slice.
- Open: whether Report Center will become an Admin projection, a Reports
  capability, or a separately approved module.
- Rejected: copying a former repository's Admin shell, authentication, RBAC,
  deployment, or directory layout into a module.
- Deferred: multi-project Analytics, expanded Reports automation, Monitoring
  alert policies and period reports, Report Center, and a fifth process.

## Development horizon and success signals

Now, consolidate onboarding, permission safety, understandable states, update
recovery, and the four retained journeys. Next, deepen only the highest-friction
steps proven by adoption evidence. Later, evaluate bounded notifications,
exports, or target-specific helpers; hosted SaaS, multi-tenancy, fleet
administration, dynamic plugins, and a broader extension model require a new
product-boundary decision.

The product is succeeding when a new owner can install, sign in, understand
health, and complete one useful journey in every enabled module; ordinary
operators see only permitted actions; module failures remain diagnosable
without losing Admin; and runs and updates retain understandable success,
failure, and recovery evidence. Quantitative targets remain open until real
usage or user feedback exists.

## New-module acceptance

Before implementation begins, a module proposal must provide:

1. a fixed live source revision for every former repository used as evidence;
2. a capability matrix with `retain`, `reproduce`, `reuse`, `extend`, `wrap`,
   `new`, `defer`, or `drop` for every selected behavior;
3. declared product, data, permission, and navigation ownership;
4. a bounded user journey with normal and failure acceptance;
5. a reuse search through the current shared Rust and frontend owners;
6. verification for the selected behavior and every new shared contract;
7. synchronized architecture, project map, product, UI, command, and deployment
   documentation when those boundaries change.

## Ready verdict

Ready for legacy comparison and feature-specification slices.

Not ready for direct implementation of multi-project Analytics, expanded
Reports automation, Report Center, or a fifth module. Each remains deferred
until its named feature specification resolves behavior, permissions,
migration, failure states, and acceptance.
