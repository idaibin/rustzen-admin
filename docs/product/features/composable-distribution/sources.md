# Community evidence and applied decisions

Retrieved 2026-09-03. These sources support mechanisms and tradeoffs, not proof
that Rustzen has implemented them. Proposed defaults and topology choices are
our design decisions. No external source overrides the user's scope.

| Source | Supported fact | Applied decision / limit |
| --- | --- | --- |
| [OpenTelemetry Collector Builder](https://opentelemetry.io/docs/collector/extend/ocb/) | An explicit component manifest can generate a custom distribution | Resolve selected owners before compilation and packaging; no need to adopt OTel itself |
| [Cargo features](https://doc.rust-lang.org/cargo/reference/features.html) | Features are additive; feature unification and combination testing matter | Positive optional dependencies, explicit minimal builds and per-selection graph verification |
| [Cargo resolver](https://doc.rust-lang.org/cargo/reference/resolver.html), [cargo tree](https://doc.rust-lang.org/cargo/commands/cargo-tree.html) | Resolver output and actual compilation are different evidence; feature union remains relevant | Retain explicit resolver 2 and inspect each binary's actual owner closure |
| [Cargo targets](https://doc.rust-lang.org/cargo/reference/cargo-targets.html), [build scripts](https://doc.rust-lang.org/cargo/reference/build-scripts.html) | Targets and compile-time scripts affect compiled products and generated inputs | Explicit binary targets and inventories for cfg/env/generated files/native links |
| [Docker Compose profiles](https://docs.docker.com/compose/how-tos/profiles/) | Profiles selectively activate services | Useful runtime operation, insufficient evidence of artifact/schema/frontend exclusion |
| [Transactional outbox](https://microservices.io/patterns/data/transactional-outbox.html) | Commit business data and event atomically; relay may duplicate | Producer outbox plus consumer persistent idempotency, with an explicit capacity/horizon bound |
| [Prometheus alerting overview](https://prometheus.io/docs/alerting/latest/overview/) | Alert evaluation and notification handling are distinct responsibilities | Monitor owns incident lifecycle; optional Admin message center owns recipient/read state |
| [WHATWG SSE](https://html.spec.whatwg.org/multipage/server-sent-events.html) | EventSource options, framing, reconnect and Last-Event-ID semantics | Existing Bearer authentication uses fetch streaming; framing and recovery are tested explicitly |
| [Axum SSE](https://docs.rs/axum/latest/axum/response/sse/index.html) | Axum supports event streams and keepalive | Reuse Axum response facilities, avoid a new HTTP transport implementation |
| [Tokio broadcast](https://docs.rs/tokio/latest/tokio/sync/broadcast/index.html) | Bounded broadcast receivers can lag and lose retained values | A queue is not durable delivery; use targeted connection queues and reconciliation |
| [NGINX proxy buffering](https://nginx.org/en/docs/http/ngx_http_proxy_module.html#proxy_buffering) | Proxy buffering changes when upstream bytes reach clients | Disable buffering for SSE and test the deployed proxy's heartbeat/read-timeout behavior |
| [SQLite WAL](https://sqlite.org/wal.html) | WAL has same-host/shared-memory constraints and separate checkpoint behavior | Keep DBs local to owners, avoid network filesystems/multi-host shared writers |
| [SQLite transactions](https://www.sqlite.org/lang_transaction.html) | Only one writer; IMMEDIATE obtains a write transaction before subsequent checks | Check actor and final owner count atomically within one access mutation |
| [JWT RFC 7519](https://www.rfc-editor.org/rfc/rfc7519.html), [HTTP signatures RFC 9421](https://www.rfc-editor.org/rfc/rfc9421.html) | Token expiry and message-signature covered fields/application context need explicit validation | Hard stream expiry plus separate directional signature domains and tested canonical fields; not a claim to implement the full RFC 9421 wire format |
| [SQLite backup API](https://sqlite.org/backup.html) | Live DB backup requires a consistent snapshot mechanism | No plain-copy live `.db` backup; restoring data is a separate recovery-point decision |
| [SQLx Migrator](https://docs.rs/sqlx/latest/sqlx/migrate/struct.Migrator.html) | Migration validation is part of executing a configured migration set | Generate a deterministic fresh baseline, fail on schema mismatch rather than suppress validation |
| [TanStack Router code splitting](https://tanstack.com/router/latest/docs/guide/code-splitting) | Lazy route splitting produces loadable code chunks | Exclude route roots before generation; lazy loading is not physical removal |
| [TanStack file routing configuration](https://tanstack.com/router/latest/docs/api/file-based-routing) | Route directory and generated/temp output paths are configurable | Make selected-only discovery and output paths explicit per build |
| [Vite glob imports](https://vite.dev/guide/features.html#glob-import) | Glob imports are build-transformed module inputs | Audit broad imports that can reintroduce omitted modules; generate explicit imports |
| [Vite static assets](https://vite.dev/guide/assets.html#the-public-directory) | Public-directory files are copied as-is separately from imported asset handling | Assemble only selected public files and inventory their output |
| [C4 diagrams](https://c4model.com/diagrams) | Context/container/component views communicate different scopes | Use two concrete deployment diagrams and one ownership map; no abstract OS model |
| [Microservice tradeoffs](https://martinfowler.com/articles/microservice-trade-offs.html) | Independent services add distribution, consistency and operational costs | Preserve useful existing failure boundaries; do not split notification transport into another service |
| [systemd unit source](https://github.com/systemd/systemd/blob/main/man/systemd.unit.xml) | Unit dependency/ordering semantics are separately defined | Generate only selected units and distinguish ordering from failure coupling |

The systemd hosted manual returned 403 during this research; the upstream
documentation source was available. Exact target-host systemd behavior still
requires native integration tests. External documentation accessed through
`latest` or `main` must be pinned to implementation dependency versions when
coding starts; it is not a compatibility promise.

## Follow-up references

Installation references checked during round 8:

- [systemd service semantics](https://github.com/systemd/systemd/blob/main/man/systemd.service.xml)
  and [unit dependencies](https://github.com/systemd/systemd/blob/main/man/systemd.unit.xml):
  recovery needs explicit requirement, ordering and finite timeout, without
  waiting for dependent services from inside its own startup job.
- [Linux openat2](https://man7.org/linux/man-pages/man2/openat2.2.html): beneath
  and no-symlink resolution support descriptor-relative privileged extraction.

Browser lifecycle references checked during round 7:

- [Fetch Standard](https://fetch.spec.whatwg.org/): abort and response handling
  must be implemented explicitly for a fetch-based event stream.
- [Chrome Page Lifecycle](https://developer.chrome.com/docs/web-platform/page-lifecycle-api):
  frozen pages cannot promise timely callbacks; restore must revalidate state.

Persistence follow-up references checked during round 5:

- [SQLx query validation](https://docs.rs/sqlx/0.9.0/sqlx/macro.query.html):
  macro checks depend on the supplied schema/metadata; dynamic queries still
  need selected-schema integration coverage. This matches the current 0.9 line.
- [SQLite URI options](https://www.sqlite.org/uri.html) and
  [schema inventory](https://www.sqlite.org/schematab.html): use no-create,
  read-only admission and observed objects rather than trusting an identity row.
  Do not treat immutable mode as safe for a live or changing WAL database.

## Method

1. Start with concrete user journeys and a measurable absence definition.
2. Map current source and deployment coupling before choosing abstractions.
3. Compare a minimal refactor with realistic alternatives and retain explicit
   rejected-option reasons.
4. Specify data ownership, trust boundaries and crash windows before interfaces.
5. Validate selected distributions with positive and negative tests.
6. Use external reviews to challenge a fixed candidate; reconcile every finding
   against source and primary references before changing the design.

This is a bounded engineering method. It does not justify new services, layers,
plugins, dimensions, business values or frameworks merely to fill a template.
