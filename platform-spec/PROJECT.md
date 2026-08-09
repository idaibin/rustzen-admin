# Project Baseline

## Project

| Field | Value |
| --- | --- |
| Repository | `rustzen-admin` Web/Rust monorepo |
| Working root | Repository root |
| Product boundary | Admin, Monitor, Insights, Reports in one signed release |
| Platform-spec owner | Daibin (repository owner) |
| Fixed basis | Pre-change source basis: `feat/platform-spec-workflow` @ `5c0479bb4f586434440eca050a66526b2021d68c` |
| Derived package | Current worktree changes layered on the fixed source basis; uncommitted package files are not attributed to that `HEAD` |
| Baseline state | `Automated` for named checks run against the derived worktree package result; runtime remains separately bounded |

## Boundaries

- `apps/admin/` owns identity, RBAC, role definitions, module control, and
  Admin-owned HTTP behavior.
- `apps/web/` owns the role page and its feature-local interaction states.
- `crates/auth/` owns shared capability checks and constants.
- `apps/admin/migrations/` owns the Admin SQLite schema.
- [DESIGN.md](../DESIGN.md) owns shared visual semantics only. It does not
  define role business rules or API contracts.
- `platform-spec/` records cross-cutting evidence and ownership; it does not
  become a runtime registry or generated contract.

## Evidence vocabulary

| State | Meaning |
| --- | --- |
| `Declared` | A product decision or acceptance target is recorded. |
| `Source-resolved` | Paths and symbols were checked against the recorded source basis and any explicitly listed package edits. |
| `Artifact-resolved` | A relevant build or generated artifact was inspected. |
| `Automated` | A named command passed in its recorded environment against the derived worktree package; this does not prove the package files are in `HEAD`. |
| `Runtime-resolved` | The current target runtime was exercised and observed. |
| `Gap` | Target and implementation differ, or required evidence is missing. |
| `Not verified` | The required runtime, external, or environment evidence was not run or is unavailable. |

## Requirement ledger

| ID | Requirement | Owner | Evidence |
| --- | --- | --- | --- |
| REQ-PLAT-001 | Keep this baseline Markdown-first and source-linked. | Daibin | `Automated`; validated by [verification.md](./verification.md). |
| REQ-PLAT-002 | Keep fixed-basis and runtime-evidence boundaries explicit. | Daibin | `Automated`; checked by the same command. |

## Acceptance

The project baseline is acceptable when the six fixed Markdown files exist, all
required headings and local links resolve, requirement/gap IDs are unique, and
the status vocabulary is present. This is a static documentation acceptance,
not runtime truth.
