# Optional-examples (OE) task ledger — rustzen-admin composable distribution

Active ledger for the core/full composition campaign. Definition and
evidence authority: `implementation.md` in this directory (execution log
with commits, receipts, and evidence paths); this file carries status,
Owner, blockers, and the Ready frontier only. Owner below means the
capability mutating the task's surface; execution is ZCode as the sole
implementer unless a task names another owner.

## Status summary

- Done (stage-accepted or review-accepted): OE-0, OE-1, OE-2 slice 1
  (core composition), DOT-OE-1900-01/02/04, OE-2 slice 2 (core Web),
  DOT-OE-1010-01/02, OE-2 slice 3a + signer closure (`b2de445`
  re-review accepted 2026-10-10).
- OE-T12 local installer scope independently accepted 2026-10-13: fixed
  source review and hash-verified historical positive/negative evidence.
  No dynamic re-execution or Linux acceptance is claimed.
- Current Linux scope (2026-10-24): current-source signed core/full fixtures
  build and install on the task-owned x86_64/systemd environment. Lifecycle,
  same-composition update/reverse/failure rollback, cross-composition refusal
  and 18 real interruption checkpoints have native evidence. Final independent
  ZCode acceptance passed (2026-10-24; no P0–P2); the Linux 23-case browser gate
  and local evidence publication passed (outer exit 0). See the current implementation receipt and ignored
  `.codex/artifacts/core-delivery/linux/`.
- Not verified: genuine preceding-full 0.5.1 old-worker compatibility. The
  original signed artifact could not be recovered in the searched locations;
  the user excluded recovery and its dependent OE-A09 row from this finite
  run. This remains an OE-T10/release-readiness blocker. New local fixture
  builds, including current-source 0.5.1, do not substitute for that release.
- OE-T11's earlier local full/core acceptance remains independently accepted
  (ZCode PASS, 2026-10-23). Its UI-only aggregate fixture remains separate from
  real delivery; older V2 failure evidence remains historical.

## Ready frontier

1. The finite current-fixture Linux change and native/browser acceptance are
   independently accepted (ZCode PASS, 2026-10-24; no P0–P2). Revalidate only
   affected claims if source, fixture identity or trust changes.
2. OE-T10 can resume its historical-worker row only with the exact signed
   legacy artifact and trusted verification materials in an authorized isolated
   runtime. The current native fixture campaign does not close that row.
3. Production and both goods VMs are excluded. Task-owned runtime cleanup
   completed after evidence retention; no Git commit, push or release is implied.

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
- **Blocker**: none for current-fixture Linux UI-state execution/publication;
  final independent acceptance passed under OE-T11.

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

- **Outcome**: composition-aware package script, signed marker, and update
  worker: full archive keeps the exact legacy member set (composition in
  the marker only), core archive per the new contract, verifier rejects
  composition mismatch before candidate install, backup, or switch
  (anti-misplacement).
- **Owner**: release/packaging.
- **Scope**: `scripts/package-release-bundle.sh`, `scripts/deploy-sign.mjs`,
  `apps/admin/src/features/manage/deploy/bundle/**`, `deploy/core/*`.
- **Inputs**: adopted contract (implementation.md); DOT-OE-1900-03 +
  DOT-OE-1010 constraints.
- **Dependencies**: OE-T04 (core binary/web exist).
- **Acceptance**: worker rejects cross-composition archives/markers by
  test in both compositions; legacy full artifact (real
  `target/rz/rz-0.5.1-x86_64.tar` + real key) passes the new worker's
  legacy admission; signer/packager accept an explicit composition with
  member-inventory cross-checks; real package builds remain Linux-gated
  (see OE-T10/OE-T12).
- **Evidence**: slice 3a commit (implementation.md section);
  `target/rz/oe2-slice3a-packaging-20261010/PROVENANCE.md` — real
  artifact admitted by the Rust worker (1/0) and by the signer (exit 0),
  wrong-composition claim rejected (exit 1); core 148/0/1, full 223/0/1.
  Signer closure (review package after `3f59559`): exact member rules
  unified with the Rust worker, reviewer counterexample reproduced then
  rejected both pre-signing and post-splice, focused tests 7/0 —
  `target/rz/oe3-signer-closure-20261010/PROVENANCE.md`.
- **Status**: Linux-real current-fixture packaging evidence delivered;
  independently accepted by ZCode on 2026-10-24 (no P0–P2). Core/full Docker and
  just wiring, signed inventories and native fresh installation are verified.
  Earlier signer acceptance (`b2de445`, 2026-10-10) remains historical evidence.
- **Blocker**: none for current-fixture package construction. Genuine legacy
  material/old-worker acceptance remains separately unverified under OE-T10.
  The former open limitation
  (the Rust real-artifact test returning silently on artifact absence)
  is closed as of 2026-10-12 with both implementation and dynamic
  acceptance: absence now fails loudly (fail-closed, never a skip)
  through an explicit NOT-RUN gate naming the missing inputs, and the
  missing-input counter-example was executed against the real test
  entry in an isolated key-free source copy — the target test FAILED
  on NOT-RUN and cargo exited 101 directly. Follow-up OE-LEGACY-SELECT-01
  (v1, 2026-10-12) split ordinary CI from the real-material gate: the
  test is `#[ignore]`d (ordinary `cargo test --workspace` no longer
  selects it; exact ordinary invocation verified 0 passed / 1 ignored /
  exit 0) and the single dedicated entry `just test-real-legacy`
  selection-gates the run (exactly-one list check, then execution with
  cargo's raw exit code; missing inputs FAILED on NOT-RUN with cargo
  101, and a deliberately non-matching selection was rejected non-zero
  without executing). Review finding P1 was fixed in v2 (same task,
  2026-10-12): the gate takes no arguments at all (target substitution
  and leading-dash smuggling are rejected before cargo, with fake-cargo
  markers proving zero invocations), counts with exact-line always-
  numeric awk, and preserves cargo's raw exit codes for both the list
  and the execution call; verified by a fake-cargo deterministic
  harness (12 scenarios, 19/19 assertions) plus isolated-copy runs
  recorded by a persisting wrapper — see
  `target/rz/oe-legacy-select-01-v2-20261012/PROVENANCE.md`. The
  real-input positive admission itself remains
  not executed (OE-T10 window); receipts, commands, and log hashes in
  implementation.md, `target/rz/oe-legacy-gate-20261012/PROVENANCE.md`,
  and `target/rz/oe-legacy-select-01-20261012/PROVENANCE.md`.

### OE-T10 — Real-artifact upgrade/rollback proof (DOT-OE-1900-03)

- **Outcome**: direct upgrade and rollback demonstrated with the actual
  old signed 0.5.1 artifact and its real worker; JSON-field tolerance is
  never accepted as the compatibility evidence.
- **Owner**: release/packaging + Linux runtime.
- **Scope**: update worker (`bundle.rs` chain), old artifacts under
  `target/rz/`.
- **Inputs**: OE-T09 candidates.
- **Dependencies**: OE-T09 (partially delivered), Linux environment.
- **Acceptance**: real old worker accepts/rejects exactly per contract on
  the real artifacts; rollback restores the previous current.
- **Evidence**: new-worker direction proven 2026-10-10 (real artifact
  admitted with real-key signature verification — see slice 3a
  PROVENANCE). Old-worker direction: none, and explicitly not
  substitutable — the new verifier accepting the old artifact does not
  satisfy this task.
- **Status**: Blocked only for the genuine historical-worker/material row;
  excluded from the current finite run by the user on 2026-10-24. Current
  local-fixture Linux update, reverse application, rollback and boot recovery
  evidence is available and does not complete this task.
- **Blocker**: the original `rz-0.5.1-x86_64.tar`, SHA-256
  `01acde7fa359794d933bb5e6ae37400555651b99da5675b66bbee900ff6942ad`,
  was not recovered in the recorded local/backup/release searches. The user
  reported it unavailable and instructed skipping recovery. Resume with that
  exact artifact, trusted materials and an authorized isolated runtime; never
  relabel a new current-source 0.5.1 build as the historical release. The
  dedicated `rz-admin-d03-amd64` runtime was authorized for the current run;
  environment availability is no longer the missing-input explanation.
  Production and goods environments remain outside scope. This blocks release
  readiness, not the other verified work.

### OE-T12 — Core fresh-install installer (OE-2 slice 3b)

- **Outcome**: composition-aware `deploy/setup-layout.sh` and `rz-install`
  generation: a core bundle's member whitelist, service layout, and
  config handling accept the core inventory and reject foreign members;
  local harness (`scripts/test-setup-layout.sh`) covers both shapes.
- **Owner**: release/installer shell.
- **Scope**: `deploy/setup-layout.sh`, `scripts/test-setup-layout.sh`,
  justfile installer generation.
- **Inputs**: OE-T09 core shape.
- **Dependencies**: OE-T09 (delivered scope).
- **Acceptance**: installer harness green for full and core shapes;
  unknown paths rejected in both; signature identity, the binaries, and
  the Web composition are bound together so a wrong package, stale
  assets, or tampering is rejected before any overwrite and before any
  database or service change; the legacy full member compatibility is
  preserved.
- **Evidence**: 2026-10-10 package (baseline `b6af0ee`): harness 21
  groups exit 0 (core positive + foreign-member, marker-mismatch,
  missing-member, unknown-member negatives with no-write assertions);
  the real `rz-0.5.1-x86_64.tar` installs end-to-end with the real key
  (exit 0, full composition); found and fixed the `b2de445` marker-field
  regression that had broken full installs. Receipts:
  `target/rz/oet12-installer-20261010/PROVENANCE.md`. Pending
  independent review (dot) at that time; local scope accepted on
  2026-10-13 (see the appended implementation receipt).
- **Status**: Local installer scope accepted; current-fixture native core/full
  fresh install, lifecycle/reboot and occupied-root refusal verified. Final
  Linux current-fixture acceptance passed independent ZCode review
  (2026-10-24; no P0–P2).
- **Blocker**: none for that bounded scope. Historical-worker compatibility
  remains OE-T10; it is not fresh-install evidence.

### OE-T11 — Browser DOM acceptance (core shell + restored card)

- **Outcome**: rendered-page acceptance of the core Admin shell and the
  Monitor delivery card (login → surfaces, card states).
- **Owner**: acceptance/verification.
- **Scope**: served core/full web.
- **Inputs**: OE-T04, OE-T05.
- **Dependencies**: OE-T05 (surface restored).
- **Acceptance**: DOM-level evidence (screenshots/interactions) against a
  real served instance.
- **Evidence**: 2026-10-13 existing V2 core/full module-panel DOM and
  browser responses remain historical (see A-06 receipt in
  ../../../plans/product-iteration-tasks.md); they are not current Monitor
  delivery-card evidence. The original Admin client on 2026-10-13 also
  verified `/monitoring/incidents` failure badge, incidents error/Reload,
  and HTTP 503/code 40001. In the isolated same-source window on
  2026-10-23, three synthetic agent reports were accepted (HTTP 200/code
  0), generated four Monitor incidents, and all four corresponding events
  were stored by the real local Admin ingress with recipients. The owner
  delivery API returned pending/quarantine/gaps 0 and non-null lastSuccess;
  owner and restricted-viewer DOM showed the card details. Separately,
  direct updates to aggregate counters only (no payload/receipt rows) made
  the owner and restricted-viewer UI show a 3-gap trigger, pending 2,
  quarantine 1 and first/last gap timestamps while preserving real
  lastSuccess. This fixture proves UI rendering only. The shared card's
  narrow popover was fixed and rechecked: at 390 CSS pixels its tooltip
  bounds were x=8, width=358, right=366, with readable text and Retry. Owner
  final IAB captures show owner 1440×900 English gap/details and restricted-
  viewer 390×844 Chinese gap and healthy details. Owner healthy recovery was
  separately confirmed by native real-API readback and the headless restored-
  healthy trigger; no healthy popover is claimed. Isolated controlled 403 and
  503 loading/failure/
  retry/recovery flows passed; each headless context was closed with no
  pending requests or timers. A current core-composition Admin was built
  from `dist-core` and checked through owner login, dashboard, narrow
  navigation, roles and audit reads; no Monitor/Insights/Reports/Notifications
  API requests occurred. The current core inventory includes the dashboard
  helper and excludes the notification shell. Evidence and build receipts
  are under ignored `.codex/artifacts/core-delivery/`.
- **Status**: Local full/core acceptance independently accepted (ZCode PASS,
  2026-10-23; no P0–P2). Review and closure records are in ignored
  `.codex/reviews/admin-d03-core-*` and `final-closure.json` under the
  receipt directory above. The fixed-source Linux 23-case gate and local evidence publication
  passed on 2026-10-24 (outer exit 0).
- **Blocker**: none for current-fixture Linux scope: browser publication and
  independent ZCode acceptance passed (2026-10-24; no P0–P2).
  Native current-fixture evidence is now available; genuine old-worker proof
  remains separately unverified under OE-T10. The aggregate gap fixture remains
  synthetic UI evidence, separate from real delivery.
