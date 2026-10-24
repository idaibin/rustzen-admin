# Full distribution acceptance

This matrix is the release acceptance authority for the first complete distribution.
First-install acceptance is intentionally separate from version-to-version update
acceptance: release 0.5.0 does not need a fabricated second version in order to ship.

## Optional examples acceptance

These future gates do not inherit pass status from full-only or selected-prototype
evidence. Each receipt records commit, dirty-diff hash if any, composition, target,
artifact SHA/signature identity, command, outcome, and evidence path. Final readiness
requires an immutable clean source/artifact pair and an independent review.

| Stable ID | Required scenario / expected result | Implementation/test owners |
| --- | --- | --- |
| OE-A01 | Fresh core with no module binaries, DBs, listeners or module credentials: real login, account/role RBAC, audit capture/list and layout work | Admin app/config/auth/audit + owned runtime fixture; UI dashboard slice |
| OE-A02 | Owner/admin/viewer/custom role and revoked session: API gates hold in both compositions; absent module direct URLs/API fail closed, with no dead nav or unauthorized data | auth/module router/permission tests + browser/API receipts |
| OE-A03 | Core first English, saved Chinese, invalid preference; sidebar/search/dashboard have no module requests, notification polling or tracker; full retained journeys work | shared Web build graph + actual browser at 1440x900 and 390x844; no manual route-tree edits |
| OE-A04 | Full installed/enabled/offline vs disabled vs not installed stay distinct; runtime disable/enable does not install files; unknown/absent-module enable rejected | module registry/reconciliation + gateway/UI tests |
| OE-A05 | Core needs no module config; missing included-module config/secrets and bad composition fail explicitly, without partial startup/data mutation; audit/maintenance are Admin-local | config/runtime/schema/worker integration tests |
| OE-A06 | Both composition build commands produce real supported-target binaries/Web; core archive physically lacks module code/assets/config/units; inventory matches compiled behavior | justfile/Docker/package graph and archive/binary inspection |
| OE-A07 | Wrong key, unsigned/tampered inventory, missing/extra file, bad path/mode/arch/version rejected before activation; full Controller pairing metadata remains bound | JS/Rust/shell shared verifier fixtures and real signed artifact tests |
| OE-A08 | Fresh disposable systemd install, start/stop/restart and host restart: core expects Admin only, full four services; service identities, paths, generated secrets and CLI status correct | setup-layout/CLI/service units + owned Linux runtime |
| OE-A09 | Real supported preceding full release to compatibility candidate then new full (or proven direct path): preserve users, grants, audit, module data, settings and credentials; old artifact provenance recorded | actual old binary/verifier + owner upload/apply; no resets or synthetic old-version claim |
| OE-A10 | Core→core/full→full success and failed health/restart: validate exact identity, backup included DBs and restore journaled restart set; reject wrong composition before write | deployment DB/journal/recovery tests and real candidate runtime |
| OE-A11 | Crash before/after snapshot, publish/current switch, each service restart and health gate: boot recovery restores previous link/config/units/data under previous inventory; no mixed versions | existing fault injection expanded to two compositions |
| OE-A12 | Existing full cannot silently become core from changed defaults; occupied-root installer still refuses bypass update; legacy format admission is exact signed full only | installer/update negative tests; compatibility receipts |
| OE-A13 | README EN/ZH, product/UI/architecture, manifest examples, command help and current stage agree; independent review closes findings | docs/link/command checks + final reviewer report |

Minimum full runtime proof uses the supported x86_64 Linux target. Keep each other
advertised architecture as a separate build/package/runtime row; untested rows are
Not verified and cannot be claimed release-ready. Use local candidate versions only
in clearly labeled test fixtures; public version/release approval is separate.
No unit-only or stubbed systemctl run substitutes for OE-A08–11. If the actual old
release artifact cannot be obtained, OE-A09 stays blocked. A compatibility bridge
that only accepts new candidates does not itself prove rollback to the old format.

## Required evidence

| Layer | Verification | Passing result |
| --- | --- | --- |
| Source | Focused Rust, Bun and shell tests plus `git diff --check` | Package verification, generated secrets, Monitor fresh startup and update/rollback branches pass. |
| Build | `just build-release` / `just build-release-core` | The default full command assembles five Linux executables and full Web assets; the explicit core command assembles only `rz` + `rz-admin` with core Web assets. Each writes a separate composition archive and signs/verifies its exact inventory. |
| Artifact | Signature and archive inspection | The composition-specific release signature verifies; frontend/backend digests match the embedded Web build and selected executables; the exact required member set is present. Core physically omits module binaries, module configuration, and module units. |
| Fresh install | Packaged installer in a new Linux systemd root | The command accepts only the bundle path, asks for no password or key, writes no placeholder secrets, installs one immutable release and enables `rz-full.service`. |
| First start | `rz start` | Core reports Admin only; full reports Admin, Monitor, Insights and Reports. `rz doctor` and service lifecycle output use the same compile-time composition inventory. |
| Restart | Service restart followed by host/container restart | The same release returns to four-service health without configuration or database repair. |
| Update safety | Focused Admin deployment tests | Whole-bundle verification, four-database backup, health gating, interruption recovery and rollback branches pass. |
| Privilege boundary | PID1 service identity and filesystem checks | Reports, Admin, Monitor and Insights run with their approved dedicated identities; ordinary services cannot modify release, unit or sibling data paths; privileged update work is isolated and bounded. |
| User journey | Existing current-source browser acceptance | Login, module navigation and useful Monitoring, Analytics and Automation journeys remain valid when the release contains no UI behavior change. |

For every later version, add one real upgrade run from the immediately preceding
released version. That run must prove owner upload and apply through the Admin API,
four-service health on the new version, and controlled failed-update rollback. This is
a gate for the later version, not a reason to invent a second candidate for 0.5.0.

## Security checks

- The signing private key is absent from the archive, installed files, environment and logs.
- A bundle signed by a different key, a modified bundle and an unsigned bundle fail
  before installation writes.
- Admin release routes continue to require the existing owner deployment capabilities;
  packaged signature trust does not replace API authorization.
- Runtime secrets are generated independently on the target and are not copied from
  the release archive as fixed shared values.

## Evidence boundaries

A source test does not prove a release artifact. A verified artifact does not prove
systemd startup. A disposable Linux run is target-like runtime evidence, not external
production deployment. Browser acceptance applies only to the exact runtime and bundle
identity recorded by the run.

## Release decision

The first release is ready for operator deployment after the source, build, artifact,
fresh-install, first-start, restart, update-safety and privilege-boundary rows pass and
an independent review finds no unresolved release-blocking defect. Commit, push,
publication and external-host deployment are separately authorized delivery actions.
