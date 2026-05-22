# Documentation Audit Report — rustzen-admin

> Historical snapshot. This file is not current implementation truth; use `docs/README.md`, `docs/architecture.md`, and `docs/guides/` for current documentation rules.

**Audit Date:** 2026-05-20
**Auditor:** automated review
**Scope:** pre-optimization snapshot before the documentation consolidation pass
**Snapshot Total Files:** 29 `.md` files
**Total Lines:** 2,893

---

## 1. Complete File Inventory

| # | Path | Lines | Purpose | Status | Issues |
|---|------|------:|---------|--------|--------|
| 1 | `README.md` | 96 | Entry / overview | ✅ Active | Repo tree duplicated (3×); doc list duplicated (4×) |
| 2 | `README-zh.md` | 95 | Entry / overview (Chinese mirror) | ⚠️ Redundant | 1:1 mirror of README.md; tree duplicated; doc list duplicated |
| 3 | `AGENTS.md` | 40 | Entry / agent rules | ✅ Active | Source-of-truth list duplicated (also in docs/README, README, project-map) |
| 4 | `CHANGELOG.md` | 39 | Log / changelog | ✅ Active | None |
| 5 | `LICENSE.md` | 21 | Legal | ✅ Active | None |
| 6 | `zen-server/AGENTS.md` | 40 | Entry / backend agent rules | ✅ Active | Local rules overlap with `docs/backend-guide.md` |
| 7 | `zen-web/AGENTS.md` | 27 | Entry / frontend agent rules | ✅ Active | Clean; minor overlap with `docs/frontend-guide.md` |
| 8 | `docs/README.md` | 56 | Entry / doc system map | ✅ Active | Placement rules duplicated in operating-rules.md |
| 9 | `docs/architecture.md` | 151 | Architecture / layout | ✅ Active | Repo tree duplicated (3×); document layers duplicated |
| 10 | `docs/project-map.md` | 133 | Index / code location | ✅ Active | Doc index duplicated (4×); command list duplicated (3×); Phase 1 map duplicated in rollout |
| 11 | `docs/backend-guide.md` | 104 | Guide / backend rules | ✅ Active | Related Docs section template repeated across guides |
| 12 | `docs/frontend-guide.md` | 67 | Guide / frontend rules | ✅ Active | Related Docs section template repeated |
| 13 | `docs/deployment-guide.md` | 249 | Guide / deployment | ✅ Active | Largest guide file; Related Docs template repeated |
| 14 | `docs/permission-guide.md` | 89 | Guide / permission | ✅ Active | Related Docs template repeated |
| 15 | `docs/repository-comparison.md` | 145 | Reference / cross-repo | ✅ Active | Niche; stable |
| 16 | `docs/ui/README.md` | 12 | UI design inputs | ✅ Active | None |
| 17 | `docs/agents/operating-rules.md` | 27 | Agent rules / stable | ✅ Active | Reading order duplicated (AGENTS.md, docs/README.md); placement rules duplicated (docs/README.md) |
| 18 | `docs/agents/current-iteration.md` | 52 | Agent state / iteration | ⚠️ Stale | "Recently Completed" items not cleaned (6 items done); only 1 item in queue |
| 19 | `docs/goals/product-direction.md` | 30 | Goals / product | ✅ Active | Overlaps with README overview (positioning, non-goals) |
| 20 | `docs/goals/repository-evolution.md` | 33 | Goals / repo direction | ✅ Active | Overlaps with product-direction.md |
| 21 | `docs/specs/documentation-governance.md` | 285 | Spec / doc governance | ⚠️ Archive-ready | Rollout complete; spec is now stable baseline (not "proposed") |
| 22 | `docs/plans/documentation-governance-rollout.md` | 465 | Plan / rollout | ❌ Archive | **Entire rollout completed**; 465 lines of step-by-step instructions no longer needed |
| 23 | `docs/specs/admin-foundation-phase-1.md` | 385 | Spec / Phase 1 | ✅ Active | Large; extensive ownership repetition across child specs |
| 24 | `docs/plans/admin-foundation-phase-1-rollout.md` | 139 | Plan / Phase 1 rollout | ✅ Active | "done" items mixed with "next"; some status items stale |
| 25 | `docs/specs/auth-account-baseline.md` | 103 | Spec / auth+account | ✅ Active | Ownership contract duplicated in phase-1 parent spec |
| 26 | `docs/specs/rbac-baseline.md` | 158 | Spec / RBAC | ✅ Active | Ownership contract duplicated in phase-1 parent spec |
| 27 | `docs/specs/audit-baseline.md` | 136 | Spec / audit | ✅ Active | Ownership contract duplicated in phase-1 parent spec |
| 28 | `docs/specs/system-baseline.md` | 134 | Spec / system | ✅ Active | Ownership contract duplicated in phase-1 parent spec |
| 29 | `docs/specs/runtime-baseline.md` | 86 | Spec / runtime | ✅ Active | Ownership contract duplicated in phase-1 parent spec |

---

## 2. Duplication Map

### Group A: Repository Directory Tree — repeated 3 times (~110 lines total)

| File | Lines | Context |
|------|------:|---------|
| `README.md` | 40 (L24-64) | Full tree block |
| `README-zh.md` | 39 (L25-63) | Full tree block (Chinese) |
| `docs/architecture.md` | 58 (L19-76) | Full tree block with extra detail |

**Recommendation:** Keep the detailed tree in `docs/architecture.md` only. Replace README trees with a 5-line summary + link.

### Group B: Documentation Entry List — repeated 4+ times (~80 lines total)

| File | Lines | Context |
|------|------:|---------|
| `README.md` | 11 (L69-79) | Doc entry points |
| `README-zh.md` | 11 (L68-78) | Doc entry points (Chinese) |
| `AGENTS.md` | 11 (L5-15) | Source of truth list |
| `docs/README.md` | 7 (L19-25) | Guide documents list |
| `docs/project-map.md` | 23 (L16-38) | Documentation index |

**Recommendation:** Declare `docs/README.md` as the single source-of-truth list. Other files link to it.

### Group C: Reading Order — repeated 3 times

| File | Lines | Context |
|------|------:|---------|
| `AGENTS.md` | 5 (L18-23) | Reading order |
| `docs/README.md` | 5 (L6-11) | Reading order |
| `docs/agents/operating-rules.md` | 5 (L6-11) | Reading order |

### Group D: Common Commands — repeated 3 times

| File | Lines | Context |
|------|------:|---------|
| `README.md` | 8 (L83-91) | Commands block |
| `docs/architecture.md` | 8 (L143-151) | Commands block |
| `docs/project-map.md` | 8 (L126-133) | Commands block |

### Group E: Phase 1 Ownership Contracts — repeated 6 times in specs

Each child spec (`auth-account-baseline.md`, `rbac-baseline.md`, `audit-baseline.md`, `system-baseline.md`, `runtime-baseline.md`) repeats ownership boundaries already defined in the parent `admin-foundation-phase-1.md` (385 lines).

~300 lines of ownership contract text are repeated across parent+child specs.

### Group F: Repository Boundaries — repeated 3 times

| File | Lines | Context |
|------|------:|---------|
| `AGENTS.md` | 6 (L26-32) | Repository boundaries |
| `docs/architecture.md` | 5 (L113-117) | Repository boundaries |
| `docs/architecture.md` | 13 (L97-110) | Directory responsibilities |

### Group G: "Related Docs" Template — repeated in 4 guide files

| File | Lines | Context |
|------|------:|---------|
| `docs/backend-guide.md` | 5 (L9-14) | Related Docs links |
| `docs/frontend-guide.md` | 4 (L9-13) | Related Docs links |
| `docs/deployment-guide.md` | ~5 | Related Docs links |
| `docs/permission-guide.md` | ~5 | Related Docs links |

### Group H: README-zh.md — full 1:1 mirror of README.md

95 lines mirroring 96 lines. The only difference is language. Tree, doc list, and commands are identical.

---

## 3. Files to Archive / Clean Up

| File | Lines | Reason | Action |
|------|------:|--------|--------|
| `docs/plans/documentation-governance-rollout.md` | 465 | **Rollout fully completed** — all tasks done | Archive or delete; keep spec only |
| `docs/agents/current-iteration.md` | 52 | 6 of 7 items completed; only 1 remaining task | Clean "Recently Completed" section; update status |
| `docs/specs/documentation-governance.md` | 285 | Status still says "proposed" — now active baseline | Update status field |
| `docs/plans/admin-foundation-phase-1-rollout.md` | 139 | Multiple "done" items in status snapshot | Clean done items; keep next tasks only |

**Lines recoverable by archiving/cleaning:** ~656 lines (rollout 465 + current-iteration cleanup ~30 + rollout plan cleanup ~30 + status fix ~1)

---

## 4. Optimization Items (by severity)

### 🔴 High — Significant redundancy

| # | Issue | Impact | Est. Savings |
|---|-------|--------|-------------|
| H1 | `documentation-governance-rollout.md` fully completed, 465 lines | Dead weight; confusing for new readers | **465 lines** |
| H2 | Repo tree duplicated 3× (README, README-zh, architecture) | Maintenance burden; drift risk | **~80 lines** |
| H3 | Doc entry list duplicated 4-5× | Every new doc requires 4-5 updates | **~50 lines** |
| H4 | README-zh.md 1:1 mirror | Double maintenance cost | **~95 lines** (if replaced with link) |
| H5 | Phase 1 ownership contracts duplicated across parent+5 child specs | ~300 lines repeated ownership boundaries | **~200 lines** (deduplicate children) |

### 🟡 Medium — Moderate issues

| # | Issue | Impact | Est. Savings |
|---|-------|--------|-------------|
| M1 | `current-iteration.md` has 6 completed items not cleaned | Clutter; unclear what's active | **~25 lines** |
| M2 | Reading order duplicated 3× | Drift risk when order changes | **~10 lines** |
| M3 | Common commands duplicated 3× | Drift risk when commands change | **~16 lines** |
| M4 | Repository boundaries duplicated 3× | Drift risk | **~12 lines** |
| M5 | Phase 1 rollout plan has stale "done" status items | Confusing | **~15 lines** |
| M6 | `documentation-governance.md` status still "proposed" | Misleading | **1 line** |

### 🟢 Low — Minor improvements

| # | Issue | Impact | Est. Savings |
|---|-------|--------|-------------|
| L1 | "Related Docs" template in 4 guide files | Minor boilerplate | **~15 lines** |
| L2 | `product-direction.md` overlaps with README overview | Conceptual duplication | **~10 lines** |
| L3 | `docs/goals/` has 2 files with overlapping scope | Could merge | **~20 lines** |
| L4 | `zen-server/AGENTS.md` local rules overlap backend-guide | Intended by design (thin copy) | 0 (keep) |

---

## 5. Summary Statistics

| Metric | Value |
|--------|------:|
| Total `.md` files | 29 |
| Total lines | 2,893 |
| Active files | 23 |
| Files needing cleanup | 4 |
| Files to archive/delete | 1 (governance rollout, 465 lines) |
| Duplication groups identified | 8 |
| High-severity optimization items | 5 |
| Medium-severity items | 6 |
| Low-severity items | 4 |

### Estimated Savings

| Action | Lines Saved |
|--------|----------:|
| Archive completed rollout plan | 465 |
| Replace README trees with links (2 files) | 80 |
| Deduplicate doc entry lists | 50 |
| Replace README-zh with link to English + note | 85 |
| Deduplicate child spec ownership contracts | 200 |
| Clean current-iteration completed items | 25 |
| Deduplicate commands, boundaries, reading order | 38 |
| Other minor cleanups | 45 |
| **Total estimated savings** | **~988 lines** |

### Projected Result

| | Current | After Optimization |
|---|--------:|-------------------:|
| Total lines | 2,893 | ~1,905 |
| Reduction | — | **~34%** |

---

## 6. Recommended Actions (Priority Order)

1. **Archive `docs/plans/documentation-governance-rollout.md`** — all tasks complete, single biggest win
2. **Clean `docs/agents/current-iteration.md`** — remove completed items, update next queue
3. **Update `docs/specs/documentation-governance.md` status** — from "proposed" to "active"
4. **Consolidate repo tree** — keep detailed tree in `docs/architecture.md` only; README uses summary + link
5. **Consolidate doc entry list** — declare `docs/README.md` as source of truth; others link
6. **Consolidate commands** — keep in `docs/architecture.md`; README links
7. **Evaluate README-zh.md** — either commit to maintaining it or replace with a link to README.md + a Chinese note section
8. **Slim child specs** — replace full ownership contracts with references to parent `admin-foundation-phase-1.md`
9. **Clean Phase 1 rollout plan** — remove done items from status snapshot
10. **Merge `docs/goals/` files** — combine product-direction.md and repository-evolution.md if scope overlap justifies it
