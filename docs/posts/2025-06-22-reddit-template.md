# rustzen-admin: A Modern Full-Stack Admin Template with Rust + React

Hey r/rust!

I've been working on **rustzen-admin**, a full-stack admin system template that combines Rust (Axum) with React frontend. I wanted to share it with the community and get some feedback on the architecture patterns I'm using.

## What is it?

rustzen-admin is a starter template for building admin panels and dashboards. It's designed for developers who want:

- **Rust's performance and safety** on the backend
- **Modern React ecosystem** on the frontend
- **Clean project structure** to build upon
- **Type-safe full-stack development** with mock data-driven frontend development

## Tech Stack

### Rust Backend

- **Axum** - Web framework
- **SQLx** - Async PostgreSQL with compile-time checked queries
- **Tokio** - Async runtime
- **Serde** - Serialization
- **Tower-HTTP** - Middleware for CORS, tracing, etc.

### Frontend Stack

- **React 19** - Latest React with modern features
- **TypeScript** - Type safety throughout the application
- **Vite** - Fast build tool and dev server
- **TailwindCSS** - Utility-first CSS framework
- **Ant Design Pro** - Enterprise-class UI components
- **SWR** - Data fetching with caching

## Current Features

✓ **Basic Structure** - Modular backend architecture  
✓ **Database Integration** - PostgreSQL with SQLx  
✓ **Development Setup** - Docker environment with hot reload  
✓ **API Framework** - REST endpoints with proper error handling  
✓ **Frontend Scaffold** - React app with routing and UI components  
✓ **Mock Data Endpoints** - Frontend can develop independently with realistic data  
✓ **Type Safety** - Strict alignment between frontend and backend types  
✓ **Documentation** - API docs and development guides

## Architecture Pattern

The Rust backend follows a modular pattern:

```
// Each feature module has:
features/
├── user/
│   ├── model.rs      // Data structures & validation
│   ├── repo.rs       // Database operations
│   ├── service.rs    // Business logic
│   ├── routes.rs     // HTTP handlers
│   └── mod.rs        // Module exports
```

This keeps things organized and makes testing easier. The current version includes mock data endpoints to enable rapid frontend development while the backend architecture is being finalized.

## Getting Started

```
git clone https://github.com/idaibin/rustzen-admin.git
cd rustzen-admin
cp backend/.env.example backend/.env

# Node.js 24+ recommended
cd frontend && pnpm install && cd ..

just dev  # Starts everything with hot-reload
```

## Why I Built This

I found myself setting up similar patterns for different projects:

- Basic auth structure
- CRUD operations with validation
- API documentation setup
- Development environment configuration
- **Type-safe frontend-backend integration** with mock data for parallel development
- **Modern development practices** that work well with AI tools

## Questions for the Community

1. **Architecture feedback**: Does the modular structure make sense? Any suggestions for improvement?

2. **SQLx experience**: How do you handle database migrations and schema management in your projects?

3. **Error handling**: I'm using `thiserror` for custom error types. What patterns do you prefer?

4. **Testing approach**: Any recommendations for testing Axum applications effectively?

5. **Type safety**: How do you maintain type consistency between Rust backend and TypeScript frontend in your projects?

## Links

- **GitHub**: https://github.com/idaibin/rustzen-admin
- **Docs**: Setup guides and API documentation included
- **Chinese docs**: Available for international developers

## Feedback Welcome!

This is a learning project for me, so I'd appreciate any feedback:

- Code review suggestions
- Architecture improvements
- Better patterns you've used
- Missing features that would be useful
- Real-world usage experiences

**Want to contribute?** We welcome issues and pull requests! The roadmap is community-driven.

Thanks for reading!

---

_Note: This is an early-stage template. It's functional but still evolving based on real-world usage and community feedback. The current version includes mock data to enable frontend development while backend features are being implemented._
