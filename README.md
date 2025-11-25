# 📚 rustzen-admin Documentation Center

---

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Language](https://img.shields.io/badge/lang-Rust%20%7C%20TypeScript-orange.svg)
![Status](https://img.shields.io/badge/status-Development-yellow.svg)

---

[简体中文](./README-zh.md)

> A modern full-stack admin system template built with **Rust (Axum)** and **React (Vite + Ant Design)**. Designed for performance, simplicity, and scalability.

## 🎯 Project Goals

This project aims to become a **modern admin backend template** in the Rust ecosystem, providing:

1. **Out-of-the-box**: Complete RBAC permission system and basic functionality
2. **Code Quality**: Good code structure and security
3. **Easy to Extend**: Clear modular architecture
4. **Best Practices**: Demonstrates Rust + React full-stack development patterns

---

## ⚙️ Tech Stack

| Layer        | Technology                                       |
| ------------ | ------------------------------------------------ |
| **Backend**  | Rust, Axum, SQLx, PostgreSQL, Tracing            |
| **Frontend** | React, TypeScript, Vite, Ant Design, TailwindCSS |
| **Auth**     | JWT (JSON Web Tokens)                            |
| **Tooling**  | just, pnpm                                       |

---

## 📦 Directory Structure

```
rustzen-admin/
├── src/              # Rust (Axum) API service source code
├── web/              # React (Vite) admin frontend
├── migrations/       # Database migration files
├── docs/             # Project documentation
├── Cargo.toml        # Rust dependencies configuration
├── justfile          # Project command runner
└── README.md
```

---

## 🛠️ Quick Start

### Prerequisites

-   [Rust](https://www.rust-lang.org/tools/install)
-   [Node.js](https://nodejs.org/) (v24+) and `pnpm`
-   [Just](https://github.com/casey/just) command runner

### Installation & Setup

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/idaibin/rustzen-admin.git
    cd rustzen-admin
    ```

2.  **Set up environment variables:**

    ```bash
    cp .env.example .env
    # Edit .env file with database connection information
    ```

3.  **Install dependencies:**

    ```bash
    # Install just and Rust dependencies
    cargo install just
    cargo install cargo-watch

    # Install frontend dependencies
    cd web && pnpm install && cd ..
    ```

4.  **Set up database**

    ```bash
    First, ensure you have PostgreSQL installed and running. Then set up the database:

    # Install sqlx-cli if you haven't already
    cargo install sqlx-cli --features postgres

    # Set up environment variable for database connection （using .env file）
    # Option 1: Use DATABASE_URL (recommended for sqlx-cli)
    DATABASE_URL="postgresql://username:password@localhost:5432/database_name"

    # Option 2: Use RUSTZEN_DB_URL (project default)
    RUSTZEN_DB_URL="postgresql://username:password@localhost:5432/database_name"

    # Check migration status:
    sqlx migrate info

    # Expected output:
    # 101/pending system table
    # 102/pending system relation
    # 103/pending system view
    # 104/pending system func
    # 105/pending system seed

    # Run migrations:
    sqlx migrate run

    # Expected output after successful migration:
    # 101/installed system table
    # 102/installed system relation
    # 103/installed system view
    # 104/installed system func
    # 105/installed system seed
    # Check database connection
    ```

    > **Note:** The project uses `RUSTZEN_DB_URL` by default, but `sqlx-cli` uses `DATABASE_URL`.

5.  **Start the project:**

    ```bash
    just dev
    ```

    The application will be available at `http://localhost:5173`.

---

## 📖 Project Documentation

-   [🏗️ Architecture Design](./docs/architecture.md) - System modules and technical architecture
-   [⚙️ Permission Design](./docs/permissions-guide.md) - Design and usage guide

---

## 📄 Open Source License

This project is licensed under the MIT License. See [LICENSE.md](./LICENSE.md) for details.

---

Developed by [idaibin], committed to building deployable, maintainable, and scalable Rust full-stack system engineering templates 🦀

---
