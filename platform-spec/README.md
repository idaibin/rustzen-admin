# Platform Specification

This directory is a Markdown-first, repository-local baseline for platform
coordination. It is a navigation and evidence contract, not a second source of
API, DTO, route, token, or schema truth.

## Scope

- Repository: `rustzen-admin`.
- Fixed basis (pre-change source basis): branch `feat/platform-spec-workflow` at
  `5c0479bb4f586434440eca050a66526b2021d68c`.
- Specification owner: Daibin (repository owner).
- Package status: `Automated` for the named checks recorded in
  [verification.md](./verification.md), run against the derived worktree
  package layered on that source basis.

The fixed basis is the source snapshot before this package's edits. It does not
claim that the current uncommitted Markdown, scripts, or source changes are
present in the recorded `HEAD`; those files are the derived worktree package
result being checked.

## Authority

Read the shortest authority chain for the question at hand:

1. Current behavior: source code, tests, and migrations.
2. Runtime and repository structure: [docs/architecture.md](../docs/architecture.md)
   and [docs/project-map.md](../docs/project-map.md).
3. Product behavior and acceptance: the linked product Feature Spec.
4. Shared visual semantics: [DESIGN.md](../DESIGN.md); the role UI slice remains
   [docs/ui/features/role-definition-management.md](../docs/ui/features/role-definition-management.md).
5. This directory: ownership, pre-change fixed basis, derived-package
   requirement/gap ledger, changed-file inventory, and evidence boundaries.

Platform-spec prose never overrides a more authoritative source. It records
where a decision is proven and where it is still `Not verified`.

## Files

- [PROJECT.md](./PROJECT.md): project boundary and fixed basis.
- [governance.md](./governance.md): authority, owners, states, and change rules.
- [architecture.md](./architecture.md): bounded execution path and contract edges.
- [domains/role-definition-management.md](./domains/role-definition-management.md):
  one vertical feature ledger.
- [verification.md](./verification.md): executable static checks and evidence log.

## Acceptance boundary

`just verify-platform-spec` proves only the Markdown structure, IDs, links, and
declared status vocabulary. Focused Rust tests prove the selected HTTP/SQLite
seam. Build, deployment, gateway, SSO, browser, production traffic, and
rollback behavior remain separate evidence layers and must not be inferred from
this static baseline.
