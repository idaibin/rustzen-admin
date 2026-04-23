# Repository Evolution

> Status: active near-term goals

## Current Direction

`rustzen-admin` should evolve as a clean admin foundation with explicit boundaries, not as an accumulation of unrelated features.

The repository should become easier to extend in three dimensions:

- feature growth
- deployment clarity
- documentation governance

## Near-Term Goals

- finish the monorepo naming and boundary cleanup around `zen-core/`, `zen-server/`, and `zen-web/`
- keep guide documents, goals, plans, specs, and agent docs clearly separated
- make repository entry docs predictable for both maintainers and agents
- strengthen release and deployment clarity without introducing unnecessary product complexity

## Guardrails

- do not add a second shared crate unless shared non-auth runtime logic clearly exists
- do not add new app surfaces unless the product scope actually requires them
- do not turn the repository into a process-heavy documentation system
- do not compromise clarity to preserve old compatibility paths

## Expected Outcome

- contributors can tell where new code and new docs belong
- downstream product work can start from a stable repository contract
- future cleanup work can build on explicit goals instead of ad hoc conventions
