# 🧩 rustzen-admin

> A modern, fullstack admin system template built with **Rust (Axum)** and **React (Vite + Tailwind + ProComponents)**. Designed for performance, simplicity, and future extensibility (e.g., desktop, Web3).

---

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Language](https://img.shields.io/badge/lang-Rust%20%7C%20TypeScript-orange)
![Status](https://img.shields.io/badge/status-in%20development-yellow)
![Dark Mode](https://img.shields.io/badge/ui-dark--mode--ready-black)

---

## 🚀 Project Overview

**`rustzen-admin`** is an all-in-one starter kit and architectural reference for building modern admin panels using Rust and React.  
It provides a production-ready backend API server (Axum + SQLx), a sleek frontend UI (Vite + Tailwind + Ant Design ProComponents), and optional desktop integration via Tauri.

Designed for:

- Developers who love **Rust's performance and safety**
- Teams looking for a **clean fullstack project structure**
- Makers and indie devs building **internal tools or SaaS products**
- Potential future integration into **Web3 / blockchain dashboards**

---

## ✨ Features

- ✅ Full-featured user/role/menu system with RBAC
- ✅ Built with **Axum**, **SQLx**, **PostgreSQL**
- ✅ React + Vite + TailwindCSS + Ant Design Pro
- ✅ Authentication with JWT (OAuth2 planned)
- ✅ Global dark mode support
- ✅ Logging, settings, and extensibility in mind
- ✅ Modular directory structure, production-ready
- ⚙️ CLI-compatible: fully binary deployable (no Docker required)
- 🧱 Future extensibility: WebSocket, job queues, Tauri desktop

---

## ⚙️ Tech Stack

| Layer    | Tech                                                                                                    |
| -------- | ------------------------------------------------------------------------------------------------------- |
| Backend  | Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx), PostgreSQL |
| Frontend | React, Vite, TailwindCSS, Ant Design, ProComponents                                                     |
| Auth     | JWT (OAuth2 planned)                                                                                    |
| Desktop  | [Tauri](https://tauri.app/) (optional)                                                                  |
| Logging  | Tracing + JSON logs                                                                                     |
| Tooling  | dotenv, cargo-make, optional Docker                                                                     |

---

## 📦 Directory Structure

```text
rustzen-admin/
├── backend/         # Axum + SQLx Rust API service
│   ├── src/
│   ├── migrations/
│   └── Cargo.toml
├── frontend/        # React + Vite + Tailwind + ProComponents admin UI
│   ├── src/
│   └── vite.config.ts
├── desktop/         # (Optional) Tauri desktop shell
│   └── src-tauri/
├── docker/          # (Optional) docker-compose & scripts
├── docs/            # Architecture docs & usage guides
├── scripts/         # Build/deploy utilities (bash/sh)
├── .env.example     # Sample env vars
└── README.md
```

⸻

🛠️ Getting Started

1. Backend

```
cd backend
cp .env.example .env
# configure your DB settings

# run migration
cargo install sqlx-cli
sqlx migrate run

# start server
cargo run
```

2. Frontend

```
cd frontend
npm install
npm run dev
# visit http://localhost:5173
```

⸻

📚 Functional Modules

Module Status Description
🧑‍💼 User/Auth System 🔄 Planned Login, password, JWT
🔐 Role + RBAC 🔄 Planned Role-based access control
🧭 Menu System 🔄 Planned Dynamic menus by role
⚙️ System Settings 🔄 Planned General settings config
📜 API Logs 🔄 Planned Trace, log, monitor
📁 File Upload 🔄 Planned Local & optional S3 support
📡 WebSocket Push ⏳ Optional For dashboard/live updates
🖥️ Tauri Desktop ⏳ Optional Controlled kiosk mode etc.
📄 Markdown Doc Gen 🔄 Planned Dev/usage docs output

⸻

🧱 System Architecture (Planned)

            +-----------------+        +----------------+

Browser → | React Frontend | --> | Axum API |
+-----------------+ +----------------+
↓
+---------------------+
| SQLx + PostgreSQL |
+---------------------+

⚠ Future plans: split into workspace crates for auth, core, api, desktop.

⸻

🖼️ Preview (Coming Soon)

Screenshots and live demo will be provided as the system stabilizes.
You can optionally deploy via:

# For dev-only quick demo

cargo build --release
./target/release/rustzen-admin

⸻

📌 Roadmap
• Auth system with password reset / multi-session login
• Permissions per menu / route
• System audit logs
• Frontend theme switcher
• WebSocket for real-time dashboard
• Web3 (wallet login or chain viewer)

⸻

🤝 Contributing

Contributions, suggestions, and discussions are welcome!
Please check out docs/ for detailed design specs and open issues.

⸻

📄 License

MIT

⸻

🙏 Acknowledgements
• Axum
• SQLx
• ProComponents
• Tauri

⸻

Made with ❤️ and 🦀 by [idaibin]

```

```
