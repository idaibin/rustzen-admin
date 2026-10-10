# Optional-examples (OE) task ledger — rustzen-admin composable distribution

Active ledger for the core/full composition campaign. Definition and
evidence authority: `implementation.md` in this directory (execution log
with commits, receipts, and evidence paths); this file carries status,
Owner, blockers, and the Ready frontier only. Owner below means the
capability mutating the task's surface; execution is ZCode as the sole
implementer unless a task names another owner.

## Status summary

- Done (stage-accepted): OE-0, OE-1, OE-2 slice 1 (core composition),
  DOT-OE-1900-01/02/04, OE-2 slice 2 (core Web), DOT-OE-1010-01/02.
- Ready: OE-2 slice 3 (packaging integration).
- Blocked: DOT-OE-1900-03 real-artifact proof (waits on slice 3 to produce
  candidates + Linux execution), browser DOM acceptance, Docker/Linux
  runtime verification.

## Ready frontier

1. **OE-T09 packaging integration** (below) — the only currently Ready
   task; everything else is Done or Blocked.

## Tasks

### OE-T01 — Adopt planning baseline and dependency map (OE-0/OE-1)

- **Outcome**: six planning docs adopted; dependency/compatibility map
  recorded; sole-writer rule established.
- **Owner**: docs (implementation authority).
- **Scope**: `docs/product/features/composable-distribution/*`,
  `docs/product/product.md`, `docs/architecture.md`,
  `docs/ui/features/dashboard-navigation-simplification.md`.
- **Inputs**: coordinator handoff 2026-10-09.
- **Dependencies**: none.
- **Acceptance**: baseline commit + queue receipt in implementation.md.
- **Evidence**: commit `dd81c64` (implementation.md OE-0/OE-1 sections).
- **Status**: Done.
- **Blocker**: none.

### OE-T02 — Core composition with installed-module inventory (OE-2 slice 1)

- **Outcome**: core/modules feature graph; installed inventory drives
  projections; pre-write rejection for out-of-inventory mutation;
  feature-derived packaged identity (full id byte-identical).
- **Owner**: Rust admin backend.
- **Scope**: `apps/admin/**`, `crates/config/**`.
- **Inputs**: OE-T01.
- **Dependencies**: OE-T01.
- **Acceptance**: both-composition suites green + local core vertical with
  receipts.
- **Evidence**: commit `b9c8eb1`; vertical receipts in
  `target/rz/oe2-core-vertical-20261009/` (sanitized, PROVENANCE.md).
- **Status**: Done (stage-accepted by coordinator 2026-10-10).
- **Blocker**: none.

### OE-T03 — Queue fixes DOT-OE-1900-01/02/04

- **Outcome**: core config isolation from absent modules (fields, IPC
  secret, synchronizer gating); inventory regression with a real state
  flip; evidence pointers corrected, sanitized, persisted with SHA
  provenance.
- **Owner**: Rust admin backend + evidence documentation.
- **Scope**: `crates/config/**`, `apps/admin/**`, implementation.md,
  `target/rz/{oe2-core-vertical-20261009,oe-queue-1900-20261010}/`.
- **Inputs**: queue digest of 2026-10-09 19:07.
- **Dependencies**: OE-T02.
- **Acceptance**: repro-before/after exit receipts; full negative control
  stays 101; suites green.
- **Evidence**: commit `b532439`;
  `target/rz/oe-queue-1900-20261010/EXIT-RECEIPTS.md`.
- **Status**: Done (accepted from current source by coordinator).
- **Blocker**: none.

### OE-T04 — Core Web build (OE-2 slice 2)

- **Outcome**: core preset producer with real shell and staged routes,
  two-level composition assertions, dedicated `dist-core` output,
  composition-parameterized embed, `just build-web-core`.
- **Owner**: Web build/distribution.
- **Scope**: `scripts/distribution-build-web.ts`, `distribution/*`,
  `apps/web/{vite.config.ts,package.json}`, `apps/admin/{build.rs,
  src/infra/web.rs}`, justfile.
- **Inputs**: OE-T01 contract; DOT-OE-1900-03 constraint.
- **Dependencies**: OE-T02.
- **Acceptance**: core Admin serves core Web over the real API with zero
  module/notification markers on the wire; full Web green.
- **Evidence**: commit `9c9ab2f`;
  `target/rz/oe2-slice2-core-web-20261010/PROVENANCE.md`.
- **Status**: Done (stage-accepted 2026-10-10; not overall delivery).
- **Blocker**: none.

### OE-T05 — Monitor delivery closure + composition gate (DOT-OE-1010-01)

- **Outcome**: Monitor notification delivery card restored to the exact
  promised shape; monitor-notify closure complete; the four selected-Web
  tests plus the core producer wired into `just check` via
  `verify-web-compositions`.
- **Owner**: Web shell + repo gate.
- **Scope**: `apps/web/src/routes/monitoring/incidents.tsx`, justfile.
- **Inputs**: queue digest
  `f1525abd5f011d63742dc43b6ca657b81d8a938850ceed9f33baf7199f0d7784`.
- **Dependencies**: OE-T04.
- **Acceptance**: 4/4 selected-Web tests green with no assertion relaxed;
  gate invoked by `just check`.
- **Evidence**: this boundary's commit (see implementation.md
  DOT-OE-1010 section); `target/rz/oe-1010-evidence-20261010/INDEX.md`.
- **Status**: Done.
- **Blocker**: Linux UI-state evidence scripts aligned by contract but not
  executed on this host (see OE-T11).

### OE-T06 — Final-source evidence binding (DOT-OE-1010-02)

- **Outcome**: source-bound evidence index (base HEAD + dirty digest,
  per-log SHA-256, pre/post-commit distinction, post-commit anchor).
- **Owner**: evidence documentation.
- **Scope**: `target/rz/oe-1010-evidence-20261010/`.
- **Inputs**: DOT-OE-1010-02; earlier PROVENANCE/EXIT-RECEIPTS files.
- **Dependencies**: OE-T05 (logs cover its tree).
- **Acceptance**: every gate log SHA-bound to an exact source state; no
  reuse of unrelated logs.
- **Evidence**: INDEX.md above.
- **Status**: Done.
- **Blocker**: none.

### OE-T09 — Packaging integration (OE-2 slice 3)

- **Outcome**: composition-aware package script and signed marker: full
  archive keeps the exact legacy member set (composition carried in the
  signed marker only), core archive per the new contract, verifier
  rejects composition mismatch before any write (anti-misplacement), and
  build.rs root binding moves toward authenticated correct-root checks.
- **Owner**: release/packaging.
- **Scope**: `scripts/package-release-bundle.sh`, `scripts/deploy-sign.mjs`,
  `apps/admin/src/features/manage/deploy/**`, deploy scripts.
- **Inputs**: adopted contract (implementation.md); DOT-OE-1900-03 +
  DOT-OE-1010 constraints.
- **Dependencies**: OE-T04 (core binary/web exist).
- **Acceptance**: packages build and verify-sign for both compositions;
  mismatched composition rejected pre-write by test; legacy full artifact
  (real `target/rz/rz-0.5.1-x86_64.tar` + existing key) passes legacy
  admission under the new verifier.
- **Evidence**: packaging logs + SHA receipts to be persisted per the
  1010-02 convention.
- **Status**: Ready.
- **Blocker**: none for the build/verify slice; runtime upgrade proof is
  OE-T10.

### OE-T10 — Real-artifact upgrade/rollback proof (DOT-OE-1900-03)

- **Outcome**: direct upgrade and rollback demonstrated with the actual
  old signed 0.5.1 artifact and its real worker; JSON-field tolerance is
  never accepted as the compatibility evidence.
- **Owner**: release/packaging + Linux runtime.
- **Scope**: update worker (`bundle.rs` chain), old artifacts under
  `target/rz/`.
- **Inputs**: OE-T09 candidates.
- **Dependencies**: OE-T09.
- **Acceptance**: real old worker accepts/rejects exactly per contract on
  the real artifacts; rollback restores the previous current.
- **Evidence**: to be captured (Linux execution or documented
  environment block).
- **Status**: Blocked.
- **Blocker**: needs a Linux execution environment (old worker is
  x86_64 Linux; this host is macOS/arm64) or an authorized equivalence
  proof using the real artifacts. Blocks release readiness, not slice 3.

### OE-T11 — Browser DOM acceptance (core shell + restored card)

- **Outcome**: rendered-page acceptance of the core Admin shell and the
  Monitor delivery card (login → surfaces, card states).
- **Owner**: acceptance/verification.
- **Scope**: served core/full web.
- **Inputs**: OE-T04, OE-T05.
- **Dependencies**: OE-T05 (surface restored).
- **Acceptance**: DOM-level evidence (screenshots/interactions) against a
  real served instance.
- **Evidence**: none yet.
- **Status**: Blocked.
- **Blocker**: dedicated acceptance session with a real browser against
  the served stack; explicitly not claimed by wire-level receipts.
