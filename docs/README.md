# Documentation Map

> This file is the entrypoint for the repository documentation system.

## Reading Order

1. `README.md`
2. `AGENTS.md`
3. the nearest subdirectory `AGENTS.md`
4. this file
5. the relevant guide document for the task

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

### Internal Execution Docs

Directories below are internal execution context for AI-driven continuous work and progress tracking, not default reading for contributors:

- `goals/`
- `plans/`
- `specs/`
- `agents/`

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
