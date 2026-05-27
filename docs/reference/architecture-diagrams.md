# Architecture Diagrams

These diagrams explain the current architecture. Source code and `docs/architecture.md` take precedence when details drift.

## Runtime Topology

```mermaid
flowchart LR
    Browser["Browser"] --> WebDev["Vite dev server<br/>local development"]
    WebDev --> Backend["zen-server<br/>Axum backend"]
    Browser --> BackendDeploy["zen-server<br/>packaged deployment"]
    BackendDeploy --> StaticFiles["web/dist"]
    Backend --> Db["SQLite (default)"]
    BackendDeploy --> Db
    BackendDeploy --> RuntimeData["data/uploads<br/>data/avatars<br/>logs"]
```

## Backend Request Flow

```mermaid
flowchart LR
    Route["route_with_permission"] --> Handler["handler.rs"]
    Handler --> Service["service.rs"]
    Service --> Repo["repo.rs"]
    Repo --> Db["SQLite (default)"]
    Service --> Types["types.rs"]
    Handler --> Response["ApiResponse"]
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
    Route["protected route"] --> Require["PermissionsCheck::Require"]
    Require --> Cache
    Route --> Registry["permission registry"]
    Registry --> MenuSync["startup menu sync"]
    MenuSync --> Menus["menus table"]
```
