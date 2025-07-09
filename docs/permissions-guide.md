# Flexible Permission System Guide

This guide explains how to use the enhanced permission system that supports single, any, and all permission checks through a unified API.

## Overview

The permission system provides three types of permission checks through the `PermissionsCheck` enum:

- **Single**: User must have one specific permission
- **Any**: User must have at least one of the specified permissions
- **All**: User must have all of the specified permissions

## Permission Check Types

### 1. Single Permission Check

```rust
use crate::features::auth::permission::PermissionsCheck;

// Check for a single permission
let check = PermissionsCheck::Single("system:user:list");
let has_permission = PermissionService::check_permissions(&pool, user_id, &check).await?;
```

### 2. Any Permission Check

```rust
// User needs at least one of these permissions
let check = PermissionsCheck::Any(vec![
    "system:user:create",
    "admin:full"
]);
let has_permission = PermissionService::check_permissions(&pool, user_id, &check).await?;
```

### 3. All Permissions Check

```rust
// User needs all of these permissions
let check = PermissionsCheck::All(vec![
    "system:user:delete",
    "admin:confirm"
]);
let has_permission = PermissionService::check_permissions(&pool, user_id, &check).await?;
```

## Unified Router API

The `RouterExt` trait provides a single, flexible method for registering routes with permission checks. All permission types are supported through the `PermissionsCheck` enum:

### Single Permission Routes

```rust
.route_with_permission(
    "/users",
    get(list_users),
    PermissionsCheck::Single("system:user:list")
)
```

### Any Permission Routes (OR Logic)

```rust
.route_with_permission(
    "/dashboard",
    get(dashboard),
    PermissionsCheck::Any(vec!["dashboard:view", "admin:all"])
)
```

### All Permissions Routes (AND Logic)

```rust
.route_with_permission(
    "/admin/delete",
    delete(delete_all),
    PermissionsCheck::All(vec!["admin:delete", "admin:confirm"])
)
```

## Real-World Examples

### Example 1: User Management Routes

```rust
pub fn user_routes() -> Router<PgPool> {
    Router::new()
        // Single permission for basic operations
        .route_with_permission(
            "/",
            get(get_user_list),
            PermissionsCheck::Single("system:user:list")
        )

        // Multiple valid permission paths for creation
        .route_with_permission(
            "/",
            post(create_user),
            PermissionsCheck::Any(vec!["system:user:create", "admin:full"])
        )

        // Single permission for standard operations
        .route_with_permission(
            "/{id}",
            delete(delete_user),
            PermissionsCheck::Single("system:user:delete")
        )
}
```

### Example 2: Admin Panel Routes

```rust
pub fn admin_routes() -> Router<PgPool> {
    Router::new()
        // Multiple ways to access admin dashboard
        .route_with_permission(
            "/dashboard",
            get(admin_dashboard),
            PermissionsCheck::Any(vec!["admin:dashboard", "admin:full", "super:admin"])
        )

        // Critical operations require multiple permissions
        .route_with_permission(
            "/system/reset",
            post(system_reset),
            PermissionsCheck::All(vec!["admin:system", "admin:confirm", "security:critical"])
        )
}
```

### Example 3: Complex Permission Logic

```rust
pub fn complex_routes() -> Router<PgPool> {
    Router::new()
        // Moderators or admins can access reports
        .route_with_permission(
            "/reports",
            get(get_reports),
            PermissionsCheck::Any(vec!["moderator:reports", "admin:full"])
        )

        // Financial operations require both finance and admin permissions
        .route_with_permission(
            "/finance/transactions",
            post(create_transaction),
            PermissionsCheck::All(vec!["finance:create", "admin:approve"])
        )
}
```

## Permission Cache Behavior

The permission system includes intelligent caching:

1. **Cache First**: Permissions are checked from memory cache first
2. **Auto Refresh**: Expired cache is automatically refreshed from database
3. **Re-authentication**: Missing cache requires user to log in again
4. **Logout Cleanup**: Cache is cleared when users log out

## Logging and Debugging

The system provides detailed logging for permission checks:

### Single Permission Check

```
DEBUG: Checking single permission 'system:user:list' for user 123
DEBUG: Single permission check for 'system:user:list': GRANTED
DEBUG: Permission granted: User 123 successfully validated for: single permission 'system:user:list'
```

### Any Permission Check

```
DEBUG: Checking any of permissions ["system:user:create", "admin:full"] for user 123
DEBUG: Any permission check for ["system:user:create", "admin:full"]: GRANTED (user has: ["admin:full"])
```

### All Permissions Check

```
DEBUG: Checking all permissions ["system:user:delete", "admin:confirm"] for user 123
DEBUG: All permissions check for ["system:user:delete", "admin:confirm"]: DENIED (missing: ["admin:confirm"])
WARN: Permission denied: User 123 (username: john_doe) attempted to access resource requiring: all permissions ["system:user:delete", "admin:confirm"]
```

## Best Practices

### 1. Use Appropriate Permission Types

- **Single**: For standard CRUD operations where one specific permission is sufficient
- **Any**: For operations with multiple valid authorization paths (e.g., admin override)
- **All**: For high-security operations requiring multiple approvals or confirmations

### 2. Permission Naming Convention

Follow a consistent naming pattern for permissions:

```
domain:resource:action
```

Examples:

- `system:user:list`
- `admin:full`
- `finance:transaction:create`
- `security:audit:view`

### 3. Gradual Permission Escalation

Design your routes with increasing security requirements:

```rust
// Basic operation - single permission
.route_with_permission("/users", get(list_users),
    PermissionsCheck::Single("system:user:list"))

// Sensitive operation - any of multiple permissions
.route_with_permission("/users", post(create_user),
    PermissionsCheck::Any(vec!["system:user:create", "admin:users"]))

// Critical operation - all required permissions
.route_with_permission("/users/{id}", delete(delete_user),
    PermissionsCheck::All(vec!["system:user:delete", "admin:confirm"]))
```

### 4. Error Handling

The permission system returns specific errors:

- `ServiceError::InvalidCredentials` - No valid permission cache (re-login required)
- `ServiceError::PermissionDenied` - User lacks required permissions
- `ServiceError::DatabaseQueryFailed` - Database error during permission check

## API Design Benefits

### Unified Interface

The single `route_with_permission` method handles all permission check types, making the API consistent and easy to remember.

### Type Safety

Using the `PermissionsCheck` enum ensures compile-time validation of permission logic and prevents runtime errors.

### Flexibility

The enum-based approach allows for easy extension of permission logic without breaking existing code.

### Clear Intent

Permission requirements are explicitly declared at the route level, making security requirements immediately visible.

## Migration Guide

If migrating from a string-based permission system:

### Before (if using string-based system)

```rust
.route_with_permission("/users", get(list_users), "system:user:list")
```

### After (with enum-based system)

```rust
.route_with_permission("/users", get(list_users),
    PermissionsCheck::Single("system:user:list"))
```

The new system provides the same functionality with enhanced flexibility for complex permission scenarios.
