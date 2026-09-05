# Linux runtime browser validation

This gate proves that the current full-distribution Linux binaries can start
together in one disposable Colima/Docker container and that Reports can drive a
real Chromium session through the rendered Admin Web application.

Run `just build-admin-browser-linux` and then `just verify-admin-browser-linux`.
The verifier has two bounded stages: `ensure-admin-browser-linux` builds or
reuses a pinned Debian/Chromium 120 image (900 seconds by default, capped at
1800), then the business container run is capped at 480 seconds (capped at
900). The pinned image is accepted only when its platform-specific Dockerfile
key, schema, base image, snapshot, Chromium version, OS/architecture labels,
and in-image provenance agree. Cache inspection and provenance reading are
bounded as one hot-path check. Docker architecture discovery is separately
capped at 10 seconds before either stage starts; dependency installation never
runs inside the business container. Reports also bounds the complete Chromium launch,
including CDP connection, to 30 seconds before it starts a browser flow.
The build recipe targets the Colima/Docker VM's native release architecture. It
freezes the complete source identity before Docker snapshots the context and
requires the same identity after compilation before writing provenance; a
source change during the build fails without producing trusted provenance. The verifier
requires the build provenance to match the current complete source-tree digest
and every binary hash. It copies the four binaries into a run-owned immutable
staging directory before the container starts, so a concurrent rebuild cannot
change the programs under test. The verifier
uses only the Admin gateway for authentication, Reports configuration, run
creation, status polling, and artifact download. It creates fresh SQLite files
and browser profiles under a temporary runtime root, then removes the container
and runtime data.

The browser flow signs in as the fresh-install owner, waits for the rendered
console shell, captures the Dashboard, opens Analytics details, and captures
that page. It also uses a container-only route-exact fault proxy on port 19805:
Admin setup stays direct on 19801 while the browser is routed through the proxy.
The proxy injects disconnect and HTTP failures for Reports schedule create/edit
and Monitoring global/node save/reset. The flows mutate and reread schedule and
threshold drafts; failed node reset also rereads the custom-policy marker and
reset action. Every matching mutation is counted and blocked; a duplicate or
replayed write makes the case fail instead of reaching the real backend. Python is used only for this disposable
Debian verifier because the base image has neither Bun nor Node; it is mounted
from `scripts/` and never enters the distribution. A successful run publishes exactly one evidence set under
`target/rz/ui-browser/current/` with:

- the two PNG files downloaded through the Admin artifact endpoint;
- response headers for each download;
- the build provenance binding the source tree, target, platform, distribution,
  and four binary hashes;
- a schema-2 manifest containing the Git commit, clean/dirty source state, a digest of
  the complete tracked and untracked source tree, exact binary hashes,
  immutable verifier image ID/key/provenance hash, Chromium version, byte counts,
  image dimensions, and PNG hashes. Each fault
  case records its Reports run ID, fault mode, route, screenshot hash, and dimensions.

The verifier acquires an evidence-root mutex, invalidates `current/` before
startup, and writes into a candidate directory. A concurrent invocation fails
without touching the active run. It publishes the candidate with one rename only after every service,
browser, API, PNG, dimension, and hash assertion passes. Occupied ports, stale
processes, request timeouts, failed runs, malformed artifacts, and cleanup leaks
fail closed and leave no current evidence.

The source identity is sampled before startup and checked again before publish;
any source change during the run invalidates the candidate. A dirty run remains
auditable through its source-tree digest, while release evidence should be
regenerated from the clean commit that contains the tested implementation.

This is evidence for the disposable native-architecture Linux four-service topology, embedded
Web assets, fresh-install authentication, the two named routes, and the Reports
browser/artifact path. Its success set also covers System Status module-log
diagnostics: it appends one current-day Admin marker, adds one expired Monitor
fixture while retaining the service-created current-day logs, drives the owner
Tail, backup-summary, cleanup-preview, confirmation, and result states, and
captures the specified desktop and narrow screenshots without horizontal
overflow. Archive bytes and SHA-256 remain validated by the service/client gate,
not by a browser download surrogate. It does not establish systemd PID 1
behavior, an upgrade, a production identity provider, a remote host, another
browser engine, or full visual acceptance of every route and state.
