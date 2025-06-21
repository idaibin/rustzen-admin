# 📡 API Documentation

This document outlines the API conventions for the `rustzen-admin` project, including endpoint structure, response formats, and authentication methods.

---

## Base URL

All API endpoints are prefixed with `/api`. During development, the frontend vite server will proxy requests from `/api` to the backend server (e.g., `http://localhost:8000`).

- **Production Base URL**: `https://yourdomain.com/api`
- **Development Base URL**: `http://localhost:5173/api` (proxied)

---

## 🔑 Authentication

Most endpoints require authentication via a JSON Web Token (JWT). The token must be included in the `Authorization` header of your request:

```
Authorization: Bearer <your_jwt_token>
```

Requests without a valid token to protected endpoints will receive a `401 Unauthorized` response.

Refer to the [**Authentication Guide**](./auth.md) for details on how to obtain a token.

---

## 📦 Standard Response Structure

All API responses follow a standardized JSON structure to ensure consistency and predictable handling on the frontend.

```typescript
interface ApiResponse<T> {
  code: number;
  message: string;
  data: T;
}
```

### Fields

| Field     | Type     | Description                                                                                                          |
| :-------- | :------- | :------------------------------------------------------------------------------------------------------------------- |
| `code`    | `number` | **Status Code**. `0` indicates success. Any other value indicates an error.                                          |
| `message` | `string` | **Response Message**. A human-readable message, typically "success" for successful requests or an error description. |
| `data`    | `T`      | **Response Payload**. The actual data returned by the endpoint. Can be an object, an array, or `null`.               |

### Example

#### Success Response (`code: 0`)

```json
{
  "code": 0,
  "message": "success",
  "data": {
    "userId": 1,
    "username": "admin"
  }
}
```

#### Error Response (`code: 40001`)

```json
{
  "code": 40001,
  "message": "Invalid username or password",
  "data": null
}
```

---

## Endpoints

_This section should be populated with details for each API endpoint as they are developed._

### User Management (`/api/sys/user`)

- **GET `/api/sys/user`**: Get a paginated list of users.
- **POST `/api/sys/user`**: Create a new user.
- **PUT `/api/sys/user/:id`**: Update an existing user.
- **DELETE `/api/sys/user/:id`**: Delete a user.

---

_This document is a living document and should be updated as the API evolves._
