---
title: "Rust + React Admin Template with Axum, SQLx and Vite – Open Source"
published: true
description: "A starter template for admin systems built with Rust (Axum) and React, designed for clean architecture and developer productivity."
tags: rust, react, webdev, programming
cover_image: https://dev-to-uploads.s3.amazonaws.com/uploads/articles/q2cq0nzh8mzh5zi62zgj.png
---

## Introducing

Building admin panels often involves setting up the same foundational patterns: authentication, user management, CRUD operations, and API documentation. After working on several such projects, I decided to create **rustzen-admin** - a starter template that combines Rust backend with React frontend.

## 🎯 The Motivation

Every time I started a new project requiring an admin interface, I found myself:

- Setting up basic authentication flows
- Implementing standard CRUD operations
- Configuring development environments
- Writing API documentation
- Organizing project structure

This repetitive setup work inspired me to build a template that provides a solid foundation while demonstrating modern development practices.

## 🛠️ Technology Stack

### Backend: Rust + Axum

I chose **Rust** for the backend to leverage its performance and type safety:

- **[Axum](https://github.com/tokio-rs/axum)** - Web framework with good ergonomics
- **[SQLx](https://github.com/launchbadge/sqlx)** - Compile-time checked SQL queries with PostgreSQL
- **[Tokio](https://tokio.rs/)** - Async runtime for handling requests
- **[Serde](https://serde.rs/)** - JSON serialization/deserialization

### Frontend: React + Modern Tooling

For the frontend, I used current React ecosystem tools:

- **React 19** - Latest React version
- **TypeScript** - Type safety throughout the application
- **Vite** - Fast build tool and dev server
- **TailwindCSS** - Utility-first CSS framework
- **Ant Design Pro** - UI component library
- **SWR** - Data fetching with caching

## 🏗️ Project Structure

The template follows a clean, modular architecture:

```
rustzen-admin/
├── backend/         # Rust (Axum) API Service
├── frontend/        # React (Vite) Admin UI
├── docker/          # Docker configuration files
├── docs/            # Project documentation
├── justfile         # Command runner
└── README.md
```

### Backend Architecture

Each feature module follows a consistent pattern:

```rust
features/
├── user/
│   ├── model.rs      // Data structures & validation
│   ├── repo.rs       // Database operations
│   ├── service.rs    // Business logic
│   ├── routes.rs     // HTTP handlers
│   └── mod.rs        // Module exports
```

This separation makes the code:

- **Testable** - Each layer can be tested independently
- **Maintainable** - Clear boundaries between responsibilities
- **Extensible** - Easy to add new features

## ✨ Current Features

### 🔧 Development Setup

- Docker-based development environment
- Hot reload for both frontend and backend
- Unified command runner with `justfile`
- Environment configuration management

### 🗃️ Backend Foundation

- Modular architecture with feature-based organization
- PostgreSQL database integration via SQLx
- CORS and logging middleware
- Structured error handling
- **Mock data endpoints** for rapid frontend development

### 🎨 Frontend Scaffold

- React application with TypeScript
- Component library integration (Ant Design Pro)
- Routing system setup
- State management foundation
- **Type-safe API integration** with SWR for data fetching

### 🔄 Type Safety & Development Experience

- **Strict type alignment** between frontend and backend
- **Mock data-driven development** - frontend can develop independently with realistic data
- **Compile-time safety** - TypeScript and Rust catch errors early
- **AI-friendly codebase** - clean structure works well with modern development tools

### 📚 Documentation

- API documentation and testing examples
- Development setup guides
- Architecture documentation

## 🚀 Getting Started

The setup process is straightforward:

```bash
# Clone the repository
git clone https://github.com/idaibin/rustzen-admin.git
cd rustzen-admin

# Set up environment variables
cp backend/.env.example backend/.env

# Install frontend dependencies (Node.js 24+ recommended)
cd frontend && pnpm install && cd ..

# Start development environment
just dev
```

The `just dev` command will:

1. Start PostgreSQL with Docker Compose
2. Start the Rust backend with hot reload
3. Start the React frontend with Vite
4. Open browser to `http://localhost:5173`

## 🎨 Code Examples

### Type-Safe API Contracts

Frontend and backend share the same type definitions, ensuring consistency:

```rust
// Backend: Rust types
#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}
```

```typescript
// Frontend: TypeScript types (auto-generated or manually synced)
interface User {
  id: string;
  username: string;
  email: string;
  created_at: string;
}
```

### Error Handling Pattern

```rust
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {id}")]
    NotFound { id: Uuid },
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}
```

### Repository Pattern

```rust
#[async_trait]
pub trait UserRepository {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, UserError>;
    async fn create(&self, user: CreateUser) -> Result<User, UserError>;
}
```

## 🔮 What's Next

This template provides a foundation that can be extended with:

- JWT authentication implementation
- Role-based access control
- File upload functionality
- Advanced UI components
- Testing coverage
- Database migration from mock data to real persistence

**Want to contribute?** We welcome issues and pull requests! The roadmap is community-driven.

## 🤝 Contributing

The project is MIT licensed and open to contributions. Areas where help would be appreciated:

- Code review and architecture feedback
- Documentation improvements
- Testing strategies
- Feature suggestions
- Real-world usage feedback

## 🎉 Conclusion

rustzen-admin is a starting point for building admin systems with Rust and React. It's not a complete solution but rather a foundation that demonstrates clean architecture patterns, modern tooling integration, and type-safe full-stack development.

The current version includes mock data endpoints to enable rapid frontend development while the backend architecture is being finalized. This approach allows teams to work in parallel and iterate quickly.

If you're looking to start an admin project with Rust backend, this template might save you some initial setup time while providing a structure to build upon.

**Links:**

- [GitHub Repository](https://github.com/idaibin/rustzen-admin)
- [Documentation](https://github.com/idaibin/rustzen-admin/tree/main/docs)

What do you think? Have you worked with similar tech stacks? I'd love to hear your experiences and suggestions!

---

_Happy coding! 🦀⚛️_
