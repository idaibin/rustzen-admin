---
title: "rustzen-admin Series (Part 1): Authentication Security Upgrade - From bcrypt to Argon2"
published: true
description: "A comprehensive guide to upgrading password security in Rust web applications - migrating from bcrypt to Argon2 with JWT middleware implementation."
tags: rust, security, authentication, webdev
cover_image:
---

> JWT Middleware Design + Argon2 Password Security + Complete Authentication Flow Implementation

## 🎯 Introduction: Why Upgrade Password Security?

When building enterprise-level management systems, authentication security is often the first line of defense that developers encounter. However, many projects still rely on older password hashing algorithms like bcrypt, which, while secure, may not represent the current best practices for password security.

In the **rustzen-admin** project, I recently went through a comprehensive authentication security upgrade. To be honest, I was initially hesitant about migrating from bcrypt to Argon2 - after all, bcrypt was working perfectly fine, so why fix what isn't broken? But after diving deep into modern password security standards and seeing some eye-opening vulnerability reports, I decided to bite the bullet and make the upgrade.

This article documents my entire journey - the research, the implementation challenges I faced, and the solutions I found. I hope it saves you some of the debugging time I spent staring at cryptic error messages at 2 AM.

### Why This Upgrade Matters

- **Security Enhancement**: Argon2 is the winner of the Password Hashing Competition and offers superior resistance to various attack vectors
- **Performance Optimization**: Better tunable parameters for different deployment scenarios
- **Future-Proofing**: Adopting industry-standard recommendations for password security
- **Architecture Improvement**: Implementing clean separation between authentication logic and business logic

## 🔐 Part 1: Understanding Argon2 vs bcrypt

### The Limitations of bcrypt

While bcrypt has served the industry well for over two decades, it has some inherent limitations:

```rust
// Traditional bcrypt approach (what we're moving away from)
use bcrypt::{hash, verify, DEFAULT_COST};

fn hash_password_bcrypt(password: &str) -> Result<String, bcrypt::BcryptError> {
    hash(password, DEFAULT_COST)
}

fn verify_password_bcrypt(password: &str, hash: &str) -> bool {
    verify(password, hash).unwrap_or(false)
}
```

**bcrypt Limitations:**

- **Memory Usage**: Limited memory-hard properties
- **Parallel Resistance**: Vulnerable to GPU-based attacks
- **Parameter Tuning**: Limited customization options
- **Algorithm Age**: Designed in 1999, before modern attack vectors

### Argon2 Advantages

Argon2 addresses these limitations with three variants:

- **Argon2d**: Maximum resistance against GPU attacks
- **Argon2i**: Maximum resistance against side-channel attacks
- **Argon2id**: Hybrid approach (recommended for most use cases)

## 🛠️ Part 2: Implementing the Argon2 Password Module

Here's where things got interesting. I initially tried to just swap out bcrypt calls with Argon2, but quickly realized I needed a more thoughtful approach. After some trial and error (and a few failed compilation attempts), here's the clean implementation I settled on:

```rust
// backend/src/core/password.rs
use crate::common::api::ServiceError;
use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};

/// Password utilities for secure hashing and verification.
pub struct PasswordUtils;

impl PasswordUtils {
    /// Hashes a plain-text password using Argon2.
    ///
    /// This function generates a random salt and uses Argon2 with default parameters
    /// to create a secure hash of the provided password.
    pub fn hash_password(password: &str) -> Result<String, ServiceError> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| ServiceError::PasswordHashingFailed)?
            .to_string();
        Ok(password_hash)
    }

    /// Verifies a password against a hash.
    ///
    /// This function parses the stored hash and verifies if the provided
    /// plain-text password matches the hash.
    pub fn verify_password(password: &str, hash: &str) -> bool {
        let parsed_hash = match PasswordHash::new(hash) {
            Ok(h) => h,
            Err(_) => return false,
        };
        Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok()
    }
}
```

### What I Learned During Implementation

The biggest "aha!" moment came when I realized how much simpler the error handling could be with proper type design:

1. **Salt Generation**: I initially tried to manage salts manually (bad idea). Using `SaltString::generate(&mut OsRng)` was much cleaner and more secure.

2. **Error Handling**: This took me a while to get right. I wanted all password-related errors to flow through our existing `ServiceError` system, but Argon2's error types didn't map cleanly. The solution was creating a specific `PasswordHashingFailed` variant.

3. **Default Parameters**: I spent way too much time tweaking Argon2 parameters initially. Turns out the defaults are perfectly fine for most use cases - sometimes simpler is better.

4. **Memory Safety**: One of the reasons I love Rust - I don't have to worry about accidentally leaving password data in memory. The ownership system handles cleanup automatically.

### Comprehensive Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_password_hashing_and_verification() {
        let password = "test_password_123";

        // Test hashing
        let hash = PasswordUtils::hash_password(password).expect("Should hash password");
        assert!(!hash.is_empty());

        // Test verification with correct password
        assert!(PasswordUtils::verify_password(password, &hash));

        // Test verification with incorrect password
        assert!(!PasswordUtils::verify_password("wrong_password", &hash));
    }

    #[test]
    fn test_same_password_produces_different_hashes() {
        let password = "same_password";

        let hash1 = PasswordUtils::hash_password(password).expect("Should hash password");
        let hash2 = PasswordUtils::hash_password(password).expect("Should hash password");

        // Due to random salt, same password should produce different hashes
        assert_ne!(hash1, hash2);

        // But both should verify correctly
        assert!(PasswordUtils::verify_password(password, &hash1));
        assert!(PasswordUtils::verify_password(password, &hash2));
    }
}
```

## 🔒 Part 3: JWT Authentication Middleware Design

Now for the fun part - the JWT middleware. I'll be honest, this is where I made my biggest mistake initially. I tried to implement token verification directly in each route handler. After copy-pasting the same token extraction logic for the third time, I realized I needed a proper middleware approach.

### Middleware Architecture

```rust
// backend/src/features/auth/middleware.rs
use crate::{
    common::api::{AppError, ServiceError},
    core::jwt,
};
use axum::{extract::Request, http::header, middleware::Next, response::Response};

pub async fn auth_middleware(request: Request, next: Next) -> Result<Response, AppError> {
    let (mut parts, body) = request.into_parts();

    // Extract Bearer token from Authorization header
    let token = parts
        .headers
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or_else(|| AppError::from(ServiceError::InvalidCredentials))?;

    // Verify JWT token and extract claims
    let claims = jwt::verify_token(token).map_err(|_| ServiceError::InvalidToken)?;

    // Inject claims into request extensions for downstream handlers
    parts.extensions.insert(claims);

    let request = Request::from_parts(parts, body);

    Ok(next.run(request).await)
}
```

### JWT Utility Functions

Pro tip: I initially put these functions directly in the middleware file, but quickly learned that separating JWT logic into its own module makes testing much easier:

```rust
// backend/src/core/jwt.rs (key excerpts)
use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub exp: usize,
    pub iat: usize,
}

pub fn generate_token(user_id: i64, username: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let exp = (now + Duration::seconds(JWT_CONFIG.expiration)).timestamp() as usize;
    let iat = now.timestamp() as usize;

    let claims = Claims { user_id, username: username.to_string(), exp, iat };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(JWT_CONFIG.secret.as_bytes()))
}

pub fn verify_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS256);
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_CONFIG.secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
```

## 🔄 Part 4: Complete Authentication Flow Implementation

This is where everything comes together. I had to refactor the existing auth service to use our new password utilities, and honestly, it was more work than I initially expected. The tricky part was maintaining backward compatibility during the migration.

### User Registration with Enhanced Security

```rust
// backend/src/features/auth/service.rs (key excerpts)
impl AuthService {
    pub async fn register(
        pool: &PgPool,
        request: RegisterRequest,
    ) -> Result<RegisterResponse, ServiceError> {
        tracing::info!("Attempting to register new user.");

        // Check for conflicts
        if UserRepository::find_by_username(pool, &request.username)
            .await
            .map_err(|e| {
                tracing::error!("DB error checking username: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?
            .is_some()
        {
            return Err(ServiceError::UsernameConflict);
        }

        // Hash password using new Argon2 implementation
        let password_hash = PasswordUtils::hash_password(&request.password)?;

        let new_user = UserRepository::create(
            pool,
            &request.username,
            &request.email,
            &password_hash,
            None, // real_name
            1,    // status
        )
        .await
        .map_err(|e| {
            tracing::error!("DB error creating user: {:?}", e);
            ServiceError::DatabaseQueryFailed
        })?;

        // Generate JWT token
        let token = jwt::generate_token(new_user.id, &new_user.username)
            .map_err(|e| {
                tracing::error!("Failed to generate token: {:?}", e);
                ServiceError::DatabaseQueryFailed
            })?;

        Ok(RegisterResponse {
            user: UserInfo { id: new_user.id, username: new_user.username },
            token,
        })
    }
}
```

### Login Verification with Argon2

Here's a confession: I initially forgot to update the login verification logic and spent an embarrassing amount of time wondering why all login attempts were failing. Don't make my mistake - remember to update both registration AND login!

```rust
pub async fn verify_login(
    pool: &PgPool,
    username: &str,
    password: &str,
) -> Result<UserEntity, ServiceError> {
    let user = UserRepository::find_by_username(pool, username)
        .await
        .map_err(|_| ServiceError::DatabaseQueryFailed)?
        .ok_or(ServiceError::InvalidCredentials)?;

    if user.status == 0 {
        return Err(ServiceError::InvalidOperation("User is disabled".to_string()));
    }

    // Use new Argon2 verification
    if PasswordUtils::verify_password(password, &user.password_hash) {
        UserRepository::update_last_login(pool, user.id)
            .await
            .map_err(|_| ServiceError::DatabaseQueryFailed)?;
        Ok(user)
    } else {
        Err(ServiceError::InvalidCredentials)
    }
}
```

## 🔧 Part 5: Integration with Axum Framework

The Axum integration was surprisingly smooth once I figured out the right way to structure the route layers. The key insight was understanding that middleware order matters - a lot.

```rust
// backend/src/core/app.rs (key excerpts)
pub async fn create_server() -> Result<(), Box<dyn std::error::Error>> {
    let pool = create_default_pool().await?;

    // Define public and protected routes
    let public_api = Router::new().nest("/auth", public_auth_routes());

    let protected_api = Router::new()
        .nest("/auth", protected_auth_routes())
        .nest("/system", system_routes())
        .route_layer(middleware::from_fn(auth_middleware)); // Apply middleware here

    let app = Router::new()
        .route("/", get(root))
        .nest("/api", public_api.merge(protected_api))
        .layer(cors)
        .with_state(pool);

    // Server startup logic...
    Ok(())
}
```

### Protected Route Example

One thing I love about this approach is how clean the route handlers become. The middleware does all the heavy lifting, and your handlers just focus on business logic:

```rust
// backend/src/features/auth/routes.rs
async fn get_user_info_handler(
    State(pool): State<PgPool>,
    Extension(claims): Extension<Claims>, // Injected by middleware
) -> AppResult<Json<ApiResponse<UserInfoResponse>>> {
    let response = AuthService::get_user_info(&pool, claims).await?;
    Ok(ApiResponse::success(response))
}
```

## 📊 Part 6: What I Learned About Security and Performance

### Security Wins (and a few close calls)

I'll be honest - some of these I got right by accident, others I had to learn the hard way:

1. **Salt Uniqueness**: Argon2 handles this automatically, which is great because I initially tried to manage salts manually (rookie mistake).

2. **Timing Attack Resistance**: This was a happy accident - Argon2's verification is naturally constant-time, unlike some naive string comparison approaches I've seen.

3. **Memory Security**: Rust's ownership system saved me here. In other languages, I'd be paranoid about password strings lingering in memory.

4. **Token Expiration**: I learned to make this configurable after hardcoding a 1-hour expiration and getting locked out of my own app during testing.

5. **Error Information**: I initially returned detailed error messages (helpful for debugging, terrible for security). Now I return generic "invalid credentials" messages.

### Performance Optimization

```rust
// Configuration for different environments
impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: std::env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: 10,
            min_connections: 1,
            connect_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
        }
    }
}
```

### Migration Strategy (Lessons from the Trenches)

If you're migrating an existing system like I did, here's what actually worked (after a few false starts):

1. **Dual Support**: Temporarily support both bcrypt and Argon2
2. **Gradual Migration**: Hash new passwords with Argon2, verify old ones with bcrypt
3. **User-Triggered Updates**: Re-hash passwords during login
4. **Monitoring**: Track migration progress and performance impact

## 🎯 Wrapping Up: Was It Worth It?

Short answer: absolutely. Long answer: it was more work than I expected, but the peace of mind is worth it. Here's what this whole journey taught me:

1. **Security upgrades don't have to be scary**: With the right approach, you can upgrade critical systems without breaking everything.

2. **Modern tools make things easier**: Argon2 is actually simpler to use than bcrypt once you get the hang of it.

3. **Architecture matters**: Taking time to design clean interfaces (like our middleware) pays off in maintainability.

4. **Rust is your friend**: The type system caught so many potential bugs during this migration that would have been runtime errors in other languages.

### What We've Achieved

- ✅ **Enhanced Security**: Migrated to Argon2 password hashing
- ✅ **Robust Middleware**: Implemented JWT authentication middleware
- ✅ **Clean Architecture**: Separated authentication concerns
- ✅ **Comprehensive Testing**: Added unit and integration tests
- ✅ **Performance Optimization**: Improved hash performance
- ✅ **Future-Proofing**: Adopted industry standards

### What's Next?

I'm already thinking about the next improvements:

- **Multi-Factor Authentication**: TOTP support is on my roadmap
- **Session Management**: Refresh tokens would be nice for better UX
- **Rate Limiting**: Need to add brute-force protection
- **Audit Logging**: Better security event tracking

📎 **All code from this article is available in the rustzen-admin repository. Key authentication modules:**

- [core/password.rs](https://github.com/idaibin/rustzen-admin/blob/main/backend/src/core/password.rs) - Argon2 password hashing implementation
- [core/jwt.rs](https://github.com/idaibin/rustzen-admin/blob/main/backend/src/core/jwt.rs) - JWT utility functions
- [features/auth/service.rs](https://github.com/idaibin/rustzen-admin/blob/main/backend/src/features/auth/service.rs) - Authentication business logic
- [features/auth/middleware.rs](https://github.com/idaibin/rustzen-admin/blob/main/backend/src/features/auth/middleware.rs) - JWT authentication middleware

🔗 **Complete Source Code**: [rustzen-admin on GitHub](https://github.com/idaibin/rustzen-admin/tree/main/backend)

🚀 **Next in Series**: Part 2 will dive into "Enterprise-Level Rust Backend Architecture: Elegant Implementation of Repository-Service-Routes Three-Tier Pattern" - where we'll explore how to build scalable, maintainable backend systems with proper separation of concerns.

Stay tuned for more insights from the rustzen-admin project!
