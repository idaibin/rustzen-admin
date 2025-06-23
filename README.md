# 🧩 rustzen-admin

> A modern, full-stack admin system template built with **Rust (Axum)** and **React (Vite + Ant Design)**. Designed for performance, simplicity, and extensibility.

[简体中文](./README_zh-CN.md)

---

![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)
![Language](https://img.shields.io/badge/lang-Rust%20%7C%20TypeScript-orange.svg)
![Status](https://img.shields.io/badge/status-in%20development-yellow.svg)

---

## 🚀 Project Overview

**`rustzen-admin`** is an all-in-one starter kit and architectural reference for building modern admin panels. It provides a production-ready backend API server built with Rust (Axum & SQLx) and a sleek frontend UI powered by React (Vite & Ant Design).

It's designed for:

- Developers who love **Rust's performance and safety**.
- Teams looking for a **clean, modern full-stack project structure**.
- Makers building **internal tools, dashboards, or SaaS products**.

---

## ✨ Features

- ✅ **Full-Stack Framework**: A complete, integrated solution with a Rust backend and React frontend.
- ✅ **Modern Tech Stack**: Built with **Axum**, **SQLx**, **Vite**, **TailwindCSS**, and **Ant Design ProComponents**.
- ✅ **Authentication**: Comes with a ready-to-use JWT-based authentication flow.
- ✅ **Database Ready**: Integrated with PostgreSQL via SQLx, including a command-line migration setup.
- ✅ **Containerized**: Includes a multi-stage `Dockerfile` and `docker-compose.yml` for easy development and deployment.
- ✅ **Efficient Tooling**: A `justfile` provides streamlined commands (`dev`, `build`, `clean`) for the entire project.
- ✅ **Extensible by Design**: A modular structure makes it easy to add new features, WebSocket endpoints, or even a Tauri desktop app.

---

## ⚙️ Tech Stack

| Layer        | Technology                                                                                               |
| :----------- | :------------------------------------------------------------------------------------------------------- |
| **Backend**  | Rust, [Axum](https://github.com/tokio-rs/axum), [SQLx](https://github.com/launchbadge/sqlx), PostgreSQL  |
| **Frontend** | React, TypeScript, Vite, TailwindCSS, [Ant Design ProComponents](https://procomponents.ant.design/), SWR |
| **Auth**     | JWT (JSON Web Tokens)                                                                                    |
| **Tooling**  | `just`, `pnpm`, Docker                                                                                   |
| **Desktop**  | [Tauri](https://tauri.app/) (Optional)                                                                   |

---

## 📦 Directory Structure

```text
rustzen-admin/
├── backend/         # Rust (Axum) API Service
├── frontend/        # React (Vite) Admin UI
├── desktop/         # (Optional) Tauri Desktop Shell
├── docker/          # Docker configuration files
├── docs/            # Project documentation
├── justfile         # Command runner for the project
└── README.md
```

---

## 🛠️ Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (v24+) with `pnpm`
- [Just](https://github.com/casey/just) command runner
- [Docker](https://www.docker.com/get-started) (for the database)

### Installation & Launch

1.  **Clone the repository:**

    ```bash
    git clone https://github.com/idaibin/rustzen-admin.git
    cd rustzen-admin
    ```

2.  **Set up environment variables:**
    Copy the example file for the backend, which contains the `DATABASE_URL`.

    ```bash
    cp backend/.env.example backend/.env
    ```

3.  **Install frontend dependencies:**

    ```bash
    cd frontend && pnpm install && cd ..
    ```

4.  **Launch the project:**
    The simplest way to start the database, backend, and frontend together is using `just`.

    ```bash
    # This command uses docker-compose to start the database
    # and then starts the backend and frontend with hot-reloading.
    just dev
    ```

    The application will be available at `http://localhost:5173`.

---

## 🧪 API Testing

We recommend using **VSCode REST Client** for API testing:

1. **Install the plugin**: Search for "REST Client" in VSCode extensions
2. **Open test file**: `docs/api/api.http`
3. **Send requests**: Click "Send Request" above any HTTP request
4. **View responses**: Results appear in a new tab

**Key benefits**:

- ✅ Integrated with VSCode
- ✅ Version controlled with Git
- ✅ Perfect for individual development
- ✅ No additional software needed

See the complete guide: [`docs/api/rest-client.md`](docs/api/rest-client.md)

---

## 📚 Feature Implementation Status

### 🔐 Authentication System ✅ **Implemented**

| Feature             | Status      | Description                                                   |
| ------------------- | ----------- | ------------------------------------------------------------- |
| User Login          | ✅ Complete | JWT token authentication with encrypted password verification |
| User Registration   | ✅ Complete | Username/email conflict detection                             |
| Get User Info       | ✅ Complete | Includes role information and menu permissions                |
| JWT Auth Middleware | ✅ Complete | Automatic token verification and user status checking         |
| Password Hashing    | ✅ Complete | bcrypt secure password storage                                |

### 🧑‍💼 System Management ✅ **Core Features Implemented**

| Module              | Status      | Description                                         |
| ------------------- | ----------- | --------------------------------------------------- |
| **User Management** | ✅ Complete | CRUD operations, role assignment, status management |
| **Role Management** | ✅ Complete | Role CRUD, menu permission assignment               |
| **Menu Management** | ✅ Complete | Tree-structured menus, permission control           |
| **Data Dictionary** | ✅ Complete | Dictionary item management, options API             |
| **Operation Logs**  | ✅ Complete | System log recording and querying                   |

### 🔗 Options API ✅ **Implemented**

| Feature                   | Status      | Description                                       |
| ------------------------- | ----------- | ------------------------------------------------- |
| Unified Options Interface | ✅ Complete | `/api/system/{resource}/options`                  |
| Permission Control        | ✅ Complete | Atomic permission design (`system:roles:options`) |
| Search & Filtering        | ✅ Complete | Support for `q`, `limit`, `status` parameters     |
| Response Format           | ✅ Complete | Standard `{ label, value }` format                |

### 🏗️ Architecture Features ✅ **Implemented**

| Feature                    | Status      | Description                                          |
| -------------------------- | ----------- | ---------------------------------------------------- |
| **Modular Architecture**   | ✅ Complete | Repository-Service-Routes three-tier architecture    |
| **Unified Error Handling** | ✅ Complete | `ServiceError` enum with business error code mapping |
| **API Response Format**    | ✅ Complete | Unified `ApiResponse<T>` wrapper                     |
| **Database Migration**     | ✅ Complete | Complete database schema and relationships           |
| **Internationalization**   | ✅ Complete | Full English comments and error messages             |
| **Logging System**         | ✅ Complete | Tracing framework with structured logging            |
| **CORS Configuration**     | ✅ Complete | Cross-origin support, development-friendly           |

---

## 🔄 Planned Features

| Module                 | Status      | Description                                       |
| ---------------------- | ----------- | ------------------------------------------------- |
| 📁 **File Upload**     | 🔄 Planned  | Support for local/S3 upload                       |
| ⚙️ **System Settings** | 🔄 Planned  | General configuration management, theme switching |
| 📡 **WebSocket Push**  | ⏳ Optional | Real-time notifications and messaging             |
| 🖥️ **Tauri Client**    | ⏳ Optional | Desktop admin interface                           |
| 📊 **Dashboard**       | 🔄 Planned  | Data statistics and visualization charts          |
| 🌍 **Multi-language**  | 🔄 Planned  | i18n internationalization framework               |
| 🎨 **Theme System**    | 🔄 Planned  | Dark mode, custom themes                          |

---

## 🧱 Technical Architecture

```text
            +--------------------+       +------------------+
   Client → | React Admin UI     | →     | Rust API (Axum) |
            +--------------------+       +------------------+
                      ↓                           ↓
            +--------------------+       +------------------+
            | Ant Design Pro     |       | SQLx + PostgreSQL|
            | TailwindCSS        |       | JWT + bcrypt     |
            | SWR + TypeScript   |       | tracing + tokio  |
            +--------------------+       +------------------+
```

**Key Advantages**:

- 🦀 **Rust Backend**: Memory safety, high performance, type safety
- ⚛️ **React Frontend**: Modern component library, responsive design
- 🗄️ **PostgreSQL**: ACID transactions, strong consistency
- 🔐 **RBAC Permissions**: Role-based access control
- 📖 **Complete Documentation**: API docs, architecture explanation

---

## 📖 Project Documentation

- [🏗️ Architecture Design](./docs/architecture.md) - System modules and technical architecture
- [📋 API Documentation](./docs/api/) - Complete interface documentation
- [⚙️ Options API](./docs/api/options-api.md) - Dropdown options interface specification
- [🛠️ Development Guide](./docs/development/) - Contributing guide and development standards

---

## 🎯 Project Goals

This project aims to become a **modern admin template** in the Rust ecosystem, providing:

1. **Ready to Use**: Complete RBAC permission system and basic features
2. **Production Ready**: Enterprise-grade code quality and security standards
3. **Easy to Extend**: Clear modular architecture for easy feature extension
4. **Best Practices**: Showcase best practices for Rust + React full-stack development

---

## 🤝 Contributing

Contributions are welcome! Please read our [**Contributing Guide**](./docs/development/CONTRIBUTING.md) to get started.

---

## 📄 License

This project is licensed under the MIT License. See the [LICENSE.md](./LICENSE.md) file for details.

---

🙏 **Acknowledgments**
• [Axum](https://github.com/tokio-rs/axum) - Modern Rust web framework
• [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
• [Ant Design Pro](https://procomponents.ant.design/) - Enterprise-class frontend components
• [Tauri](https://tauri.app/) - Cross-platform desktop application framework
• All Rust and React open-source projects and developers in the community 🙌

---

Made with ❤️ and 🦀 by **idaibin**.
