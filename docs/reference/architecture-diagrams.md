# Architecture Diagrams

These diagrams explain the current architecture. Source code and `docs/architecture.md` take precedence when details drift.

## Runtime Topology

```mermaid
flowchart LR
    Browser["Browser"] --> Admin["rz-admin<br/>0.0.0.0:9801"]
    Vite["Vite dev server"] --> Admin
    Admin --> Web["web/dist"]
    Admin --> AdminDb["admin.db"]
    Admin -->|"loopback + HMAC"| Monitor["rz-monitor<br/>127.0.0.1:9802"]
    Admin -->|"loopback + HMAC"| Insights["rz-insights<br/>127.0.0.1:9803"]
    Admin -->|"loopback + HMAC"| Reports["rz-reports<br/>127.0.0.1:9804"]
    Monitor --> MonitorDb["monitor.db"]
    Insights --> InsightsDb["insights.db"]
    Reports --> ReportsDb["reports.db"]
    Agent["optional rz-monitor-agent"] --> Admin
```

All four server processes are members of one `rz-full.service` and one signed release
bundle, but each has its own restart and database boundary.

## Module Contract And Gateway Flow

```mermaid
flowchart LR
    RustRoute["ModuleRouter registration<br/>method + path + access + capability"] --> Axum["module Axum router"]
    RustRoute --> Manifest["runtime Manifest"]
    Manifest --> Sync["Admin background sync"]
    Sync --> Transaction["menu/capability transaction"]
    Transaction --> Registry["immutable in-memory registry"]
    Request["module API request"] --> Auth["in-memory permission cache"]
    Auth --> Registry
    Registry --> Client["reused HTTP client"]
    Client --> Delegate["request-scoped HMAC delegation"]
    Delegate --> Verify["module verifies context + local route"]
    Verify --> Handler["handler → service → repo"]
```

## Frontend API Flow

```mermaid
flowchart LR
    Page["routes/*"] --> ApiModule["src/api/<domain>/api.ts"]
    ApiModule --> Request["src/api/request.ts"]
    Request --> Backend["/api/*"]
    ApiModule --> Types["types.d.ts"]
```

## Permission Flow

```mermaid
flowchart LR
    Login["login"] --> Cache["permission cache"]
    Mutation["role/menu/module mutation"] --> Cache
    Manifest["validated module Manifest"] --> MenuSync["transactional menu sync"]
    MenuSync --> Menus["module menu rows + overrides"]
    Menus --> Navigation["runtime navigation"]
    Route["protected route"] --> Cache
    Route --> Registry["in-memory module route registry"]
```
