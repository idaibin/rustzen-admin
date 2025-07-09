# ✍️ Git Commit Convention

This document defines the Git commit message convention for the `rustzen-admin` project. The goal is to improve log readability, enable automated CHANGELOG generation, and provide context for AI-assisted tools.

The convention is based on [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) and is customized for this project with modular scopes.

---

## Format

```
<type>(<scope>): <subject>
```

---

## Commit Types

| Type       | Description                                 |
| ---------- | ------------------------------------------- |
| `feat`     | New features                                |
| `fix`      | Bug fixes                                   |
| `docs`     | Documentation only                          |
| `style`    | Formatting, spacing, etc.                   |
| `refactor` | Code refactoring (no new features or fixes) |
| `test`     | Adding or modifying tests                   |
| `chore`    | Build process, tooling, dependencies        |
| `perf`     | Performance improvements                    |
| `ci`       | CI/CD configuration and scripts             |
| `build`    | Build system or external dependencies       |
| `revert`   | Revert previous commits                     |

---

## Scope

`scope` describes the area affected by the commit, such as a feature module or layer.

| Scope   | Corresponding Module/Directory                  |
| ------- | ----------------------------------------------- |
| `api`   | Backend API                                     |
| `user`  | User management module                          |
| `role`  | Role management module                          |
| `auth`  | Authentication                                  |
| `ui`    | Frontend UI changes                             |
| `types` | Type definitions                                |
| `deps`  | Dependency updates (e.g., `deps(frontend)`)     |
| `infra` | Build, deployment, CI/CD tools (Infrastructure) |
| `docs`  | Documentation updates                           |

---

## Subject

The `subject` is a brief description of the commit. Follow these rules:

-   **Use imperative mood**: e.g., use `add` not `added` or `adds`.
-   **Start with lowercase**: No need to capitalize the first word.
-   **No ending period**: Do not end with a `.`
-   **Be concise**: Recommended under 50 characters.

---

## ✅ Commit Examples

-   **Feature**: `feat(user): add user role assignment logic`
-   **Bug fix**: `fix(api): correct pagination query in user list`
-   **Docs**: `docs(readme): update development startup instructions`
-   **Style**: `style(ui): adjust table spacing and button size`
-   **Refactor**: `refactor(auth): simplify jwt middleware injection`
-   **Dependency**: `chore(deps): bump sqlx to 0.7.1`

---
