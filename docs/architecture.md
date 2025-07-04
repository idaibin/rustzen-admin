# 🏗️ rustzen-admin Architecture Overview

**rustzen-admin** is a modern, full-stack admin system template built with Rust (Axum) for the backend and React (Vite + Ant Design) for the frontend. The project aims to provide a production-ready, extensible, and maintainable admin platform, supporting modular business logic, RBAC, and rapid feature iteration.

---

## 📁 System Modules (Backend)

| Module | Status    | Submodules/Features         | Description                       |
| ------ | --------- | --------------------------- | --------------------------------- |
| user   | ✅ Stable | CRUD, password reset        | User management, roles assignment |
| role   | ✅ Stable | CRUD, permission binding    | RBAC core, role management        |
| menu   | ✅ Stable | CRUD, permission control    | Menu structure, route control     |
| dict   | ✅ Stable | CRUD, enum config           | System dictionary, config options |
| log    | ✅ Stable | Login/operation logs        | Audit, debugging, traceability    |
| auth   | ✅ Stable | JWT, middleware, extractors | Authentication, permission checks |

**Directory structure example:**

```
backend/src/features/
├── auth/
│   ├── extractor.rs
│   ├── middleware.rs
│   ├── model.rs
│   ├── permission.rs
│   ├── repo.rs
│   ├── routes.rs
│   ├── service.rs
│   └── mod.rs
├── system/
│   ├── user/
│   ├── role/
│   ├── menu/
│   ├── dict/
│   ├── log/
│   └── mod.rs
└── mod.rs
```

**Core and Common:**

- `core/`: app entry, db, jwt, password, middleware
- `common/`: API response, error handling, router extensions

---

## 🖥️ Frontend Modules (React)

| Module | Status    | Path                       | Description                |
| ------ | --------- | -------------------------- | -------------------------- |
| user   | ✅ Stable | `src/pages/system/user/`   | User management UI         |
| role   | ✅ Stable | `src/pages/system/role/`   | Role management UI         |
| menu   | ✅ Stable | `src/pages/system/menu/`   | Menu management UI         |
| dict   | ✅ Stable | `src/pages/system/dict/`   | Dictionary management UI   |
| log    | ✅ Stable | `src/pages/system/log/`    | Operation log UI           |
| auth   | ✅ Stable | `src/pages/auth/login.tsx` | Login page, authentication |
| home   | ✅ Stable | `src/pages/home/index.tsx` | Dashboard/homepage         |

**Service Layer:**

- `src/services/system/`: API services for user, role, menu, dict, log
- `src/services/auth/`: Auth API service

**State & Routing:**

- Zustand for global state (`src/stores/useAuthStore.ts`)
- React Router for navigation (`src/router.tsx`)

---

## 🔐 Authentication & RBAC

- JWT-based authentication (backend & frontend integration)
- Middleware for permission checks (backend)
- RBAC: roles, permissions, menu-based access
- Super admin logic (`zen_admin`)
- Unified API response structure

---

## 🧩 Shared Concepts & API

- Unified API response: `{ code, message, data }`
- TypeScript types for all system modules (`src/types/`)
- Options API: `/api/system/{resource}/options` for dropdowns
- Modular, extensible service and route structure

---

## 🛠️ Technical Highlights

- **Backend:** Rust, Axum, SQLx, PostgreSQL, modular service/repo/routes, error handling, middleware, in-memory permission cache
- **Frontend:** React, Vite, TypeScript, Zustand, Ant Design, TailwindCSS, modular pages/services, unified types, API abstraction
- **DevOps:** Docker, justfile, REST Client, migration scripts

---

## 🚦 Roadmap & Extension

### Current Features (v0.1.x)

- User, role, menu, dict, log management (CRUD)
- JWT authentication, RBAC, permission middleware
- Unified error handling, API response, modular codebase
- Options API for dropdowns
- Frontend/Backend type safety

### Planned / In Progress

- Department/organization management
- System settings (key-value config)
- File upload & static resource management
- System monitoring (resource, DB, etc.)
- WebSocket support for real-time features
- Tauri desktop client (optional)

### Extension Ideas

- Web3 integration (wallet login, contract interaction)
- Multi-language support (i18n)
- Theme/dark mode switching
- Plugin architecture for dynamic modules
- Approval workflow, dynamic forms

---

## 📦 Example Module Structure

**Backend:**

```
user/
├── model.rs      // Data structures
├── repo.rs       // Database logic
├── service.rs    // Business logic
├── routes.rs     // Route handlers
└── mod.rs        // Module export
```

**Frontend:**

```
src/pages/system/user/
├── index.tsx     // List page
// (form, service, hook, types as needed)
```

---
