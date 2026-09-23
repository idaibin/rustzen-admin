# Full distribution implementation plan

Status: active.

The first release has one implementation path and one acceptance boundary:

```text
build complete bundle -> copy to server -> install -> start -> four services healthy
```

## Scope

The complete bundle contains the Web application, `rz`, Admin, Monitor, Insights,
Reports, production configuration templates, the six systemd units and the installer.
It has one version, one Ed25519 signature, one update transaction and one rollback
boundary. Frontend and backend digests are inspection fields inside that one signed
identity.

The installation command accepts only the signed bundle path. It does not accept an
administrator password, signing key, verification key or database command. The build
places the public verification key into the packaged installer and runtime
configuration; the private signing key remains in the build environment. On a fresh
root, the installer generates internal service secrets, installs the immutable release,
links and enables the units, and leaves ordinary runtime overrides editable.

Admin, Monitor, Insights and Reports each create and validate their own fresh SQLite
database when they start. Monitor's migration and schema validation remain internal
implementation steps of `rz-monitor controller`; they are not separate operator work.

Updates are initiated by an authorized owner through the Admin release API. The worker
verifies one complete candidate, snapshots all four databases, installs the candidate,
switches the single `current` link, restarts the four services and checks their exact
release health. Any failed gate restores the previous link and database snapshots.
Interrupted work is handled by the existing recovery unit.

## Active work

| Order | Work | Completion condition |
| --- | --- | --- |
| 1 | Package trust and configuration | A freshly built installer verifies the matching bundle without external key input and rejects a bundle signed by another key; generated runtime configuration contains no placeholder secrets. |
| 2 | Fresh database startup | Starting each service against an empty runtime root creates and validates its database; Monitor requires no preceding CLI command. |
| 3 | Fresh system installation | In a disposable Linux systemd environment, one install command followed by `rz start` produces four healthy services on the same release version. |
| 4 | Restart recovery | Restarting the target and the container/host preserves the installed release and returns all four services to healthy state without operator repair. |
| 5 | API update and rollback | An owner can upload and apply one complete release; a deliberately failing candidate restores the previous release and all affected databases, with the outcome visible in deployment state. |
| 6 | Documentation closure | README, product contract, architecture, deployment guide and validation commands describe the same path and contain no manual key or Monitor database instructions. |

## Ownership and execution

One implementation owner changes the installer, service startup and focused tests.
The coordinator integrates the result, builds one immutable candidate and runs the
target-like installation, restart and rollback gates. One independent reviewer checks
the fixed result. Existing unrelated working-tree changes are preserved. Commit, push
and deployment to an operator-owned host remain separate actions.

## Ready verdict

Ready for the implementation and validation slices above. Release readiness is reached
only when every row passes against one immutable signed candidate. External production
deployment remains a separate environment-owner action.
