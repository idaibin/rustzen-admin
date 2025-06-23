# 🤝 Contributing to rustzen-admin

First off, thank you for considering contributing to rustzen-admin! It's people like you that make open source such a great community. We welcome any form of contribution, from documentation and bug reports to feature requests and pull requests.

## 🚀 Quickstart

Before you start, please ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (Latest stable version)
- [Node.js](https://nodejs.org/) (v24+) with `pnpm`
- [Just](https://github.com/casey/just) command runner
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

---

## 🏗️ Project Architecture

### Backend Structure

The backend follows a modular, three-tier architecture:

```
backend/src/
├── common/           # Shared utilities and API structures
├── core/             # Core functionality (app, db, jwt, password)
├── features/         # Feature modules
│   ├── auth/         # Authentication (login, register, middleware)
│   └── system/       # System management modules
│       ├── user/     # User management
│       ├── role/     # Role management
│       ├── menu/     # Menu management
│       ├── dict/     # Data dictionary
│       └── log/      # Operation logs
└── main.rs           # Application entry point
```

Each feature module follows this pattern:

```
features/module_name/
├── model.rs      # Data structures, request/response types
├── repo.rs       # Database operations (Repository pattern)
├── service.rs    # Business logic layer
├── routes.rs     # HTTP handlers and routing
└── mod.rs        # Module exports
```

### Frontend Structure

```
frontend/src/
├── assets/           # Static assets
├── layouts/          # Layout components
├── pages/            # Page components
│   └── system/       # System management pages
├── services/         # API service functions
├── types/            # TypeScript type definitions
└── main.tsx          # Application entry point
```

> 📋 **For detailed business architecture and feature modules**, see [Architecture Design](../architecture.md)

---

## 📝 Development Guidelines

### Backend (Rust) Guidelines

#### 1. Error Handling

- Use the unified `ServiceError` enum for business logic errors
- Convert `ServiceError` to `AppError` for HTTP responses
- Add proper error logging with `tracing`

```rust
pub async fn create_user(
    pool: &PgPool,
    request: CreateUserRequest,
) -> Result<UserResponse, ServiceError> {
    tracing::info!("Creating new user with username: {}", request.username);

    // Business logic here

    if some_condition {
        tracing::error!("Failed to create user: reason");
        return Err(ServiceError::DatabaseQueryFailed);
    }

    Ok(response)
}
```

#### 2. Repository Pattern

- Keep repository methods simple and focused on database operations
- Use `sqlx::query_as!` for type-safe queries when possible
- Handle database errors at the repository level

```rust
impl UserRepository {
    pub async fn find_by_id(pool: &PgPool, id: i64) -> Result<Option<UserEntity>, sqlx::Error> {
        sqlx::query_as!(
            UserEntity,
            "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL",
            id
        )
        .fetch_optional(pool)
        .await
    }
}
```

#### 3. Service Layer

- Implement business logic validation
- Combine multiple repository calls when needed
- Add comprehensive logging

#### 4. API Responses

- Always use `ApiResponse<T>` wrapper for consistency
- Use `AppResult<T>` for handler return types
- Follow REST conventions for endpoint design

### Frontend (React) Guidelines

#### 1. Component Structure

- Use functional components with hooks
- Keep components focused and single-purpose
- Use TypeScript for all components

#### 2. API Integration

- Use SWR for data fetching
- Define proper TypeScript interfaces for API responses
- Handle loading and error states

```typescript
const { data, error, isLoading } = useSWR<ApiResponse<User[]>>(
  "/api/system/users",
  fetcher
);
```

#### 3. State Management

- Use Zustand for global state
- Keep local state minimal
- Use SWR for server state

---

## 🧪 Testing Guidelines

### Backend Testing

Run tests with:

```bash
cd backend
cargo test
```

### Frontend Testing

Run tests with:

```bash
cd frontend
pnpm test
```

### Integration Testing

Use the provided API test files:

```bash
# Open in VSCode with REST Client extension
code docs/api/api.http
```

---

## 🔧 Available Commands

The project uses `just` for command management. Available commands:

```bash
# Development
just dev                # Start both backend and frontend in dev mode
just backend-dev        # Start only backend with hot reload
just frontend-dev       # Start only frontend with hot reload

# Building
just build              # Build both backend and frontend for production
just backend-build      # Build backend release binary
just frontend-build     # Build frontend production bundle

# Maintenance
just clean              # Clean all build artifacts

# Optional (Tauri desktop)
just tauri-dev          # Start Tauri desktop app in dev mode
just tauri-build        # Build Tauri desktop app

# Docker
just docker-build       # Build Docker image
just docker-push        # Push Docker image to registry
```

---

## 🛠️ Debugging and Troubleshooting

### Common Issues

#### 1. Database Connection Issues

```bash
# Check if PostgreSQL is running
docker ps

# Start database if needed
docker-compose -f docker/docker-compose.yml up db -d

# Check connection string in .env file
cat backend/.env
```

#### 2. Frontend Dependencies

```bash
# Clear node_modules and reinstall
cd frontend
rm -rf node_modules pnpm-lock.yaml
pnpm install
```

#### 3. Backend Build Issues

```bash
# Clean cargo cache
cd backend
cargo clean
cargo build
```

### Development Tips

#### 1. Hot Reload

- Backend: Uses `cargo watch` for automatic recompilation
- Frontend: Uses Vite's built-in hot reload
- Database: Use migrations for schema changes

#### 2. Logging

- Backend: Uses `tracing` crate, configured in `main.rs`
- Frontend: Use browser dev tools and `console.log`

#### 3. Database Inspection

```bash
# Connect to database directly
docker exec -it rustzen-admin-db-1 psql -U postgres -d rustzen_admin

# View logs
docker logs rustzen-admin-db-1
```

---

## ✍️ Commit Convention

All commit messages must follow the convention outlined in our [**Git Commit Convention**](./git.md). This is crucial for maintaining a clean commit history and automating changelog generation.

### Commit Format

```
<type>(<scope>): <subject>
```

### Examples

```bash
git commit -m "feat(user): add user profile page"
git commit -m "fix(api): correct pagination query in user list"
git commit -m "docs(readme): update development setup instructions"
```

### Scopes

- `api` - Backend API changes
- `user` - User management module
- `role` - Role management module
- `auth` - Authentication system
- `ui` - Frontend UI changes
- `types` - Type definition changes
- `docs` - Documentation updates
- `deps` - Dependency updates
- `infra` - Infrastructure/build changes

---

## ✅ Submitting a Pull Request

### Before Submitting

1. **Run tests**: Ensure all tests pass

   ```bash
   cd backend && cargo test
   cd frontend && pnpm test
   ```

2. **Format code**: Use the project's formatting tools

   ```bash
   cd backend && cargo fmt
   cd frontend && pnpm format
   ```

3. **Check linting**: Fix any linting issues
   ```bash
   cd backend && cargo clippy
   cd frontend && pnpm lint
   ```

### PR Process

1. Push your changes to your fork:

   ```bash
   git push origin feat/your-awesome-feature
   ```

2. Go to the original `rustzen-admin` repository and create a new Pull Request.

3. **PR Title**: Use the same format as commit messages

   ```
   feat(user): add user profile management
   ```

4. **PR Description**: Provide clear context

   - What changes were made?
   - Why were these changes necessary?
   - How should reviewers test the changes?
   - Include screenshots for UI changes

5. **Link Issues**: Reference any related issues
   ```
   Closes #123
   Related to #456
   ```

### PR Review Process

- All PRs require at least one approving review
- CI checks must pass
- Code must follow the established patterns and conventions
- Documentation should be updated for new features

---

## 🎨 Code Style

### Backend (Rust)

- **Formatting**: Use `rustfmt` with the configuration in `rustfmt.toml`
- **Linting**: Use `clippy` with the configuration in `clippy.toml`
- **Naming**: Follow Rust conventions (snake_case for functions/variables, PascalCase for types)
- **Documentation**: Add doc comments for public APIs

```bash
cd backend
cargo fmt
cargo clippy -- -D warnings
```

### Frontend (React/TypeScript)

- **Formatting**: Use Prettier with the configuration in `prettier.config.ts`
- **Linting**: Follow ESLint rules in `eslint.config.js`
- **Naming**: Use camelCase for functions/variables, PascalCase for components
- **File naming**: Use kebab-case for file names

```bash
cd frontend
pnpm format
pnpm lint --fix
```

---

## 📋 Issue Guidelines

### Bug Reports

When reporting bugs, please include:

1. **Environment details**: OS, Rust version, Node.js version
2. **Steps to reproduce**: Clear, numbered steps
3. **Expected behavior**: What should happen
4. **Actual behavior**: What actually happens
5. **Error messages**: Full error output
6. **Screenshots**: If applicable

### Feature Requests

When requesting features, please include:

1. **Use case**: Why is this feature needed?
2. **Proposed solution**: How should it work?
3. **Alternatives considered**: Other approaches you've thought of
4. **Additional context**: Any other relevant information

---

## 📚 Resources

### Learning Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Axum Documentation](https://docs.rs/axum/)
- [React Documentation](https://react.dev/)
- [Ant Design Pro](https://procomponents.ant.design/)

### Project Documentation

- [Architecture Design](../architecture.md)
- [API Documentation](../api/)
- [Options API Specification](../api/options-api.md)

---

## 🆘 Getting Help

If you need help:

1. **Check existing issues**: Someone might have faced the same problem
2. **Read the documentation**: Check the docs/ directory
3. **Ask questions**: Open a discussion or issue
4. **Join the community**: Connect with other contributors

Remember, no question is too small! We're here to help you contribute successfully.

---

Thank you again for your contribution! Every contribution, no matter how small, helps make rustzen-admin better for everyone.
