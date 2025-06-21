# 🤝 Contributing to rustzen-admin

First off, thank you for considering contributing to rustzen-admin! It's people like you that make open source such a great community. We welcome any form of contribution, from documentation and bug reports to feature requests and pull requests.

## 🚀 Quickstart

Before you start, please ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [Node.js](https://nodejs.org/) (with `pnpm`)
- [Docker](https://www.docker.com/get-started) (Optional, for database)

## 🛠️ Development Workflow

### 1. Fork & Clone

Fork the repository to your own GitHub account and then clone it to your local machine:

```bash
git clone https://github.com/<YOUR_USERNAME>/rustzen-admin.git
cd rustzen-admin
```

### 2. Setup Environment

The project uses a `.env` file for environment variables. You can start by copying the example file:

```bash
cp backend/.env.example backend/.env
```

Modify `backend/.env` to set up your database connection string and other configurations.

### 3. Run the Project

This project uses `just` as a command runner. You can find all common commands in the `justfile`.

To start the backend and frontend services in development mode simultaneously:

```bash
just dev
```

This will:

- Start the Rust backend server with hot-reloading (`cd backend && cargo watch -x run`).
- Start the Vite frontend development server (`cd frontend && pnpm dev`).

### 4. Making Changes

Create a new branch for your changes:

```bash
git checkout -b feat/your-awesome-feature
```

Now you can start making your changes to the codebase.

## ✍️ Commit Convention

All commit messages must follow the convention outlined in our [**Git Commit Convention**](./git.md). This is crucial for maintaining a clean commit history and automating changelog generation.

A quick example:

```bash
git commit -m "feat(user): add user profile page"
```

## ✅ Submitting a Pull Request

1. Push your changes to your fork:

   ```bash
   git push origin feat/your-awesome-feature
   ```

2. Go to the original `rustzen-admin` repository and create a new Pull Request.

3. Provide a clear title and description for your PR, explaining the "what" and "why" of your changes.

4. Once submitted, the project maintainers will review your code. We may suggest some changes or improvements.

## 🎨 Code Style

- **Backend (Rust)**: Please run `rustfmt` and `clippy` before committing. The configurations are in `rustfmt.toml` and `clippy.toml`.
- **Frontend (React/TS)**: Please follow the rules defined in `.eslintrc.js` and format your code using the configuration in `prettier.config.ts`.

Thank you again for your contribution!
