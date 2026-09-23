# Full distribution acceptance

This matrix is the release acceptance authority for the first complete distribution.
First-install acceptance is intentionally separate from version-to-version update
acceptance: release 0.5.0 does not need a fabricated second version in order to ship.

## Required evidence

| Layer | Verification | Passing result |
| --- | --- | --- |
| Source | Focused Rust, Bun and shell tests plus `git diff --check` | Package verification, generated secrets, Monitor fresh startup and update/rollback branches pass. |
| Build | `just build` | Five Linux executables and the Web application are assembled into one signed tar bundle. |
| Artifact | Signature and archive inspection | One release signature verifies; frontend and backend digests match the embedded Web build and five executables; the exact required member set is present. |
| Fresh install | Packaged installer in a new Linux systemd root | The command accepts only the bundle path, asks for no password or key, writes no placeholder secrets, installs one immutable release and enables `rz-full.service`. |
| First start | `rz start` | Admin, Monitor, Insights and Reports create fresh databases and report healthy with the exact bundle version. No Monitor lifecycle command is run by the operator or test harness. |
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
