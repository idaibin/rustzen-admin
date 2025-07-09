# 🏗️ rustzen-admin Architecture Overview

**rustzen-admin** is a modern, full-stack admin system template built with Rust (Axum) for the backend and React (Vite + Ant Design) for the frontend. The backend is modular, extensible, and production-ready, supporting RBAC and rapid feature iteration.

---

## 📁 Backend Module Structure (Rust)

```
src/
├── main.rs              # Application entry point
├── common/              # Shared utilities and abstractions
│   ├── api.rs           # Unified API response structures
│   ├── error.rs         # Error handling and service errors
│   ├── router_ext.rs    # Router extensions for permission middleware
│   └── mod.rs
├── core/                # Core application components
│   ├── app.rs           # Server setup and route configuration
│   ├── db.rs            # Database connection and pool management
│   ├── jwt.rs           # JWT token generation and validation
│   ├── password.rs      # Password hashing and verification
│   └── mod.rs
└── features/            # Business logic modules
    ├── auth/            # Authentication and authorization
    └── system/          # System management features
```

---

### **common/**

-   **api.rs**: Unified API response format
-   **error.rs**: Error types and handling
-   **router_ext.rs**: Router extensions for permission-based routing

### **core/**

-   **app.rs**: Application/server setup
-   **db.rs**: Database connection pool
-   **jwt.rs**: JWT token utilities
-   **password.rs**: Password hashing/verification

### **features/auth/**

-   **middleware.rs**: JWT authentication middleware
-   **service.rs**: Authentication business logic
-   **model.rs**: Auth-related data models
-   **repo.rs**: User and permission database operations
-   **permission.rs**: RBAC permission system and cache
-   **router.rs**: Auth endpoints (login, logout, user info)
-   **extractor.rs**: Request extractors for current user

### **features/system/**

-   **user/**: User management (CRUD, role assignment)
-   **role/**: Role management (CRUD, permission binding)
-   **menu/**: Menu management (CRUD, permission control)
-   **dict/**: Data dictionary (CRUD, enum config)
-   **log/**: Operation log (recording, querying)

Each submodule (user, role, menu, dict, log) typically contains:

-   **entity.rs**: Data structures
-   **dto.rs**: Data transfer objects
-   **vo.rs**: View objects
-   **repo.rs**: Database operations
-   **service.rs**: Business logic
-   **router.rs**: API endpoints

---

## 🧩 Architecture Patterns

-   **Repository-Service-Router Pattern**: Each module is organized into repository (data access), service (business logic), and router (API layer).
-   **Permission-Based Routing**: Flexible permission checks (single, any, all) via router extensions and middleware.
-   **Unified Error Handling**: Consistent error types and API responses.

---

## 🔐 RBAC Overview

-   **Users** → **Roles** → **Menus/Permissions**
-   **Permission cache** for performance
-   **Menu-based permissions** for unified access control

---

## 🛠️ Technical Highlights

-   **Rust, Axum, SQLx, PostgreSQL**
-   **JWT authentication, RBAC, modular codebase**
-   **Unified API response, error handling, and permission middleware**

---
