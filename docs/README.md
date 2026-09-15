# Docs

This is the documentation entrypoint for `rustzen-admin`.

## Source Of Truth

1. source code
2. [architecture.md](./architecture.md)
3. [guides/](./guides/)
4. [product/product.md](./product/product.md) (product scope and decisions, not implementation truth)
5. [reference/](./reference/)
6. [history/README.md](./history/README.md) (historical input and completion records)
7. [history/](./history/)

For product positioning, direction, and module-purpose decisions, use
[product/product.md](./product/product.md). Delivered behavior remains grounded
in source code, and runtime structure remains grounded in
[architecture.md](./architecture.md).

## Files

| File | Role | Value |
| --- | --- | --- |
| [product.md](./product/product.md) | Current product fact | Defines product positioning, direction, module purposes, scope, non-goals, and decision status. |
| [product/index.md](./product/index.md) | Current product index | Lists independently consumable feature specifications and readiness. |
| [composable-distribution](./product/features/composable-distribution/spec.md) | Reviewed future design | Full and physically pruned distributions, optional inbox/SSE, and contracts; implementation underway — Monitor server P1-P4, notifications P5-P7, and P8 certification for the monitor, monitor-notify and analytics selections are closed locally; full and reports selections are not certified. |
| [monitoring](./product/features/monitoring/spec.md) | Current product specification | Defines central Agent reports, node resources, alerts, incidents, retention, and daily summaries. |
| [analytics-collection-safety](./product/features/analytics-collection-safety/spec.md) | Current product specification | Defines explicit opt-in, key/origin validation, bounded event collection, and privacy-safe defaults. |
| [scheduled-report-automation](./product/features/scheduled-report-automation/spec.md) | Current product specification | Defines daily/weekly Reports scheduling with missed-occurrence skip semantics. |
| [module-log-diagnostics](./product/features/module-log-diagnostics/spec.md) | Current product specification | Defines fixed module log viewing, bounded Blob archive integrity, and preview-confirm cleanup. |
| [admin-maintenance-tasks](./product/features/admin-maintenance-tasks/spec.md) | Current product specification | Defines the bounded Admin maintenance task console surface, states, and acceptance. |
| [metric-card-visual-consistency](./product/features/metric-card-visual-consistency/spec.md) | Current product specification | Defines the shared metric-card hierarchy and route alignment rules for Dashboard, Monitoring, and Analytics. |
| [role-definition-management](./product/features/role-definition-management/spec.md) | Current product specification | Defines custom-role behavior, permission safety, failure states, and acceptance. |
| [user-role-assignment-readiness](./product/features/user-role-assignment-readiness/spec.md) | Current product specification | Defines role retrieval and assignment readiness in user dialogs. |
| [ui/index.md](./ui/index.md) | Current UI feature index | Lists independently consumable UI contracts and their product basis. |
| [analytics-collection-safety UI](./ui/features/analytics-collection-safety.md) | Current UI specification | Defines Analytics overview/detail states and collection-status presentation boundary. |
| [scheduled-report-automation UI](./ui/features/scheduled-report-automation.md) | Current UI specification | Defines Reports schedule panel/form, run linkage, states, and responsive behavior. |
| [module-log-diagnostics UI](./ui/features/module-log-diagnostics.md) | Current UI specification | Defines System Status process-log diagnostics, backup, cleanup, and partial-result states. |
| [role-definition-management UI](./ui/features/role-definition-management.md) | Current UI specification | Defines role permission-loading states, interaction, responsive behavior, and acceptance. |
| [monitoring UI](./ui/features/monitoring.md) | Current UI specification | Defines the four Monitoring route compositions, states, and responsive rules. |
| [admin-maintenance-tasks UI](./ui/features/admin-maintenance-tasks.md) | Current UI specification | Defines the maintenance task console composition and states. |
| [metric-card-visual-consistency UI](./ui/features/metric-card-visual-consistency.md) | Current UI specification | Defines metric-card layout and value-presentation alignment across routes. |
| [user-role-assignment-readiness UI](./ui/features/user-role-assignment-readiness.md) | Current UI specification | Defines role-list loading and assignment readiness states in user dialogs. |
| [message-center UI](./ui/features/message-center.md) | Current UI specification | Defines the durable inbox shell, SSE lifecycle, state matrix, and safe subject navigation. |
| [runtime-browser-validation](./ui/runtime-browser-validation.md) | Current UI gate description | Defines the disposable Linux Colima Chromium full-distribution gate and its evidence validity rule. |
| [ai-coding-rules.md](./guides/ai-coding-rules.md) | Current rule | Defines source-of-truth order, module ownership, and task verification expectations for AI-assisted changes. |
| [architecture.md](./architecture.md) | Current fact | Defines repository boundaries, runtime topology, data flow, and command source. |
| [project-map.md](./project-map.md) | Current fact | Maps important directories without implementation detail. |
| [dashboard-navigation-simplification UI](./ui/features/dashboard-navigation-simplification.md) | Current UI specification | Defines the implemented Dashboard and navigation simplification boundary. |
| [guides/backend.md](./guides/backend.md) | Current rule | Gives backend layering, naming, config, SQL, and prohibited-change rules. |
| [guides/frontend.md](./guides/frontend.md) | Current rule | Gives route, API, state, UI, and generated-file rules. |
| [guides/shared-capabilities.md](./guides/shared-capabilities.md) | Current rule | Defines shared ownership, extraction gates, reuse decisions, and former-product module intake. |
| [guides/deployment.md](./guides/deployment.md) | Current rule | Gives runtime layout, config, deploy-path, and build-output rules. |
| [guides/permission.md](./guides/permission.md) | Current rule | Gives permission ownership, route-check, menu-sync, and authorization rules. |
| [guides/automation-templates.md](./guides/automation-templates.md) | Current guide | Describes the Reports browser automation execution and evidence boundary with task templates. |
| [guides/local-verification.md](./guides/local-verification.md) | Current verification record | Records the local verification scope, evidence, and the finite remaining delivery checklist. |
| [guides/monitoring-api.md](./guides/monitoring-api.md) | Current rule | Lists the Monitor ModuleRouter API surface and response envelope rules. |
| [guides/monitoring-architecture.md](./guides/monitoring-architecture.md) | Current fact | Fixes the implementation architecture behind the Monitoring product specification. |
| [guides/monitoring-testing.md](./guides/monitoring-testing.md) | Current test matrix | Defines the executable Monitoring acceptance matrix and its database rules. |
| [reference/README.md](./reference/README.md) | Appendix index | Lists optional deep-context files. |
| [reference/architecture-diagrams.md](./reference/architecture-diagrams.md) | Appendix | Visualizes topology and request flows. |
| [reference/capability-map.md](./reference/capability-map.md) | Appendix | Maps current capabilities to real backend and frontend owners. |
| [reference/api-camelcase-audit.md](./reference/api-camelcase-audit.md) | Appendix | Audits API casing boundaries. |
| [reference/workspace-root-impl.md](./reference/workspace-root-impl.md) | Appendix | Explains runtime-root path derivation. |
| [reference/code-review-checklist.md](./reference/code-review-checklist.md) | Appendix | Provides a compact review checklist. |
| [reference/legacy-module-comparison.md](./reference/legacy-module-comparison.md) | Current comparison | Fixes live former-product revisions and maps selected behaviors to retain, reproduce, reuse, defer, or drop decisions. |
| [history/README.md](./history/README.md) | Historical index | Explains where non-current records live. |
| [history/feats/login-page-design.md](./history/feats/login-page-design.md) | Historical design | Preserves the completed login-page design input and asset link. |
| [history/feats/sqlite-first-roadmap.md](./history/feats/sqlite-first-roadmap.md) | Historical feature task record | Breaks the sqlite-first design into executable and verifiable tasks. |
| [history/plans/independent-service-refactor.md](./history/plans/independent-service-refactor.md) | Completed execution baseline | Defines the implemented four-application service split, runtime contract, permission flow, release boundary, and validation gates. |
| [history/plans/update-docs.md](./history/plans/update-docs.md) | Historical task list | Records the completed documentation-governance task request. |
| [history/fixes/documentation-audit-report-2026-05-20.md](./history/fixes/documentation-audit-report-2026-05-20.md) | Historical audit | Preserves the pre-consolidation documentation audit snapshot. |

## Placement Rules

- Put current product positioning, direction, and module-purpose boundaries in [product.md](./product/product.md).
- Put current implementation facts in `architecture.md` or `project-map.md`.
- Keep durable product boundaries and confirmed decisions in `product/product.md`; do not add a case-variant duplicate.
- Put current development rules in `guides/`.
- Put optional diagrams, audits, specs, and checklists in `reference/`.
- Put completed designs, task records, proposals, fixes, and incidents in `history/`.
- Do not put Chinese text in documentation files.
- Use `kebab-case.md` for Markdown file names.
- Ship a feature directory under `product/features/` as `spec.md` by default. A design-package layout with `architecture.md`, `contracts.md`, `implementation.md`, `review.md`, `sources.md`, and `validation.md` belongs only to a reviewed design package; `composable-distribution` is the current example. Do not copy that layout onto slices that a single specification already covers.
- sqlite-first design records under docs/history/ are historical inputs. Current implementation truth remains source code, [architecture.md](./architecture.md), and [guides/](./guides/).
