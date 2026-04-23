# Documentation Map

> This file is the entrypoint for the repository documentation system.

## Reading Order

1. `README.md`
2. `AGENTS.md`
3. the nearest subdirectory `AGENTS.md`
4. this file
5. the relevant guide, goal, plan, spec, or agent document for the task

## Document Areas

### Guides

Long-lived repository guides and technical rules:

- `architecture.md`
- `backend-guide.md`
- `frontend-guide.md`
- `deployment-guide.md`
- `permission-guide.md`
- `project-map.md`
- `repository-comparison.md`

### Goals

Long-lived repository and product direction:

- `goals/product-direction.md`
- `goals/repository-evolution.md`

Use `goals/` for:

- product direction
- repository intent
- medium- and long-lived goals

Do not use `goals/` for:

- execution steps
- task tracking
- temporary implementation notes

### Plans

Sequencing and delivery planning:

- `plans/2026-04-22-documentation-governance-rollout.md`

Use `plans/` for:

- phased rollout work
- scoped implementation plans
- active and upcoming sequencing

Do not use `plans/` for:

- stable architecture rules
- current execution state
- historical logs

### Specs

Formal design and structure contracts:

- `specs/2026-04-22-documentation-governance.md`

Use `specs/` for:

- bounded design decisions
- structure contracts
- implementation-shaping specs

Do not use `specs/` for:

- task tracking
- day-to-day execution notes

### Agents

Agent-facing stable rules and current state:

- `agents/operating-rules.md`
- `agents/current-iteration.md`

Use `agents/` for:

- stable agent operating rules
- current execution scope and exit conditions

Do not use `agents/` for:

- long-term product goals
- implementation plans
- historical run logs

## Placement Rules

- Put product direction in `goals/`.
- Put work sequencing in `plans/`.
- Put design contracts in `specs/`.
- Put stable agent rules and current execution state in `agents/`.
- Keep stable technical rules in the guide documents.

## Maintenance Rules

- Keep `AGENTS.md` thin and use it as an entrypoint, not a full manual.
- Update entry documents when document areas or names change.
- Keep one source of truth for each type of information.
- Prefer links over duplicated text when a rule already exists elsewhere.
