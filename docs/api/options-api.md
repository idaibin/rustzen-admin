# ⚙️ Options API Specification

## 1. Overview

This document defines the general-purpose "Options" API for the `rustzen-admin` project, used for frontend dropdowns, autocomplete, and form linkage scenarios.

The API provides a lightweight, unified way to fetch resource options (such as roles, users, menus), returning an array of key-value pairs in the format `[{ label, value }]`.

## 2. Design Principles

- **Unified Path**: Each resource that provides options exposes a `/options` sub-path under its standard API route.
- **Standard Structure**: The response `data` field is always an array of `OptionItem`, each containing `label` and `value`.
- **Lightweight & Efficient**: Only the minimal fields needed for frontend display and logic are returned.
- **Independent Permission**: The Options API has its own permission (e.g., `system:roles:options`), decoupled from the parent resource's CRUD permissions for fine-grained access control.

---

## 3. API Definition

### 3.1. HTTP Request

- **Method**: `GET`
- **Path Format**: `/api/system/{resource}/options`

**Path Examples:**

| Resource | API Path                    | Description          |
| -------- | --------------------------- | -------------------- |
| Role     | `/api/system/roles/options` | Get all role options |
| User     | `/api/system/users/options` | Get all user options |
| Menu     | `/api/system/menus/options` | Get all menu options |

### 3.2. Query Parameters

| Name     | Type   | Required | Description                                                                                                              | Example            |
| -------- | ------ | -------- | ------------------------------------------------------------------------------------------------------------------------ | ------------------ |
| `q`      | string | No       | Keyword for fuzzy search, usually matches the `label` field.                                                             | `?q=admin`         |
| `limit`  | number | No       | Limit the number of returned items for performance. Default is unlimited or backend-defined.                             | `?limit=10`        |
| `status` | string | No       | Filter by status. **By default, only "enabled" resources are returned.** Pass `disabled` or `all` to get other statuses. | `?status=disabled` |

### 3.3. Response Structure

Follows the project's unified `ApiResponse` format.

#### OptionItem Type

```typescript
interface OptionItem {
  label: string; // Display text
  value: string | number; // Unique identifier
  [key: string]: any; // Optional, allows extra metadata
}
```

#### Success Response Example

```json
{
  "code": 200,
  "message": "Success",
  "data": [
    { "label": "Super Admin", "value": 1 },
    { "label": "Content Editor", "value": 2 }
  ],
  "timestamp": "2024-07-30T10:00:00Z"
}
```

---

## 4. Permission Control

### 4.1. Atomic Permission Design

For fine-grained access control, the Options API should have its own atomic permission, not simply inherit the parent resource's read permission.

**Principle**: Each operation (such as fetching a dropdown list) should have a dedicated permission.

**Permission Examples:**

| Resource | Options API Permission | Full List Permission |
| -------- | ---------------------- | -------------------- |
| Role     | `system:roles:options` | `system:roles:list`  |
| User     | `system:users:options` | `system:users:list`  |
| Menu     | `system:menus:options` | `system:menus:list`  |

This design separates "viewing the management page" from "referencing the resource elsewhere".

### 4.2. Scenario Example: User Manager

Suppose a "User Manager" role is responsible for managing users, including assigning roles to users, but should not be able to manage roles themselves (CRUD roles).

In this case, assign the following permissions:

- `system:users:list`, `system:users:edit` (manage users)
- `system:roles:options` (fetch role dropdown when editing users)

They **should not** be granted `system:roles:list`, so they cannot access the "Role Management" page.

### 4.3. Backend Implementation

On the backend, middleware protecting the Options API should check for the specific `options` permission.

```rust
// Pseudocode: Check if user has permission for 'GET /api/system/roles/options'
let required_permission = "system:roles:options"; // Note: not :list
check_permission(user, required_permission).await?;
```

---

## 5. Error Responses

Error responses follow the project's unified format.

### Insufficient Permission (403 Forbidden)

```json
{
  "code": 403,
  "message": "Insufficient permission to access this resource",
  "data": null,
  "timestamp": "2024-07-30T10:00:00Z"
}
```

### Resource Not Found (404 Not Found)

This usually occurs when the `resource` path is incorrect.

```json
{
  "code": 404,
  "message": "Requested resource not found",
  "data": null,
  "timestamp": "2024-07-30T10:00:00Z"
}
```
