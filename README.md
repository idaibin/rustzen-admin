# rustzen-admin

Rust + React admin template. The backend lives in `server/`, and the frontend lives in `web/`.

## Document Entry Points

- [AGENTS.md](./AGENTS.md): root collaboration rules and repository-level conventions
- [server/AGENTS.md](./server/AGENTS.md): backend quick-entry guidance inside `server/`
- [web/AGENTS.md](./web/AGENTS.md): frontend quick-entry guidance inside `web/`
- [docs/architecture.md](./docs/architecture.md): repository-wide architecture, document layers, boundaries, and commands
- [docs/project-map.md](./docs/project-map.md): entrypoint and high-frequency path index
- [docs/backend-guide.md](./docs/backend-guide.md): backend layering, naming, database, and error rules
- [docs/frontend-guide.md](./docs/frontend-guide.md): frontend routing, state, request, and UI rules
- [docs/deployment-guide.md](./docs/deployment-guide.md): production layout, release flow, and runtime config rules
- [docs/permission-guide.md](./docs/permission-guide.md): permission model and usage rules

## Common Commands

```bash
just dev-server # start the backend only
just dev-web    # start the frontend only
just check      # backend check + frontend vp lint
just build      # build backend and frontend
```

## Document Layers

- Root documents: `README.md` and `AGENTS.md`. They define the repository entrypoint and global collaboration rules.
- Subdirectory entry documents: `server/AGENTS.md` and `web/AGENTS.md`. They define quick local rules for each subproject.
- `docs/` specification documents: six core documents covering architecture, backend, frontend, deployment, permissions, and the project map.

## Layout

```txt
.
├── server/
│   ├── Cargo.toml
│   ├── migrations/
│   └── src/
│       ├── features/
│       │   ├── auth/
│       │   ├── dashboard/
│       │   └── system/
│       ├── infra/
│       ├── common/
│       └── middleware/
├── web/
├── docs/
├── AGENTS.md
├── justfile
├── Cargo.toml
├── Cargo.lock
└── pnpm-workspace.yaml
```
