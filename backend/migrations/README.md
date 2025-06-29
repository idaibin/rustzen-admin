# Database Migrations

This directory contains database migration files for the system.

## Migration Files

### Core System

- `001_system_schema.sql` - Core database schema (users, roles, menus, associations)
- `002_system_seed.sql` - Initial system data (default roles, menus, super admin user)

### Optional Features

- `003_log_system.sql` - Log system implementation with partitioning and management functions

## Usage

### First Time Setup

```bash
# 1. Run core schema migration
psql -d your_database -f migrations/001_system_schema.sql

# 2. Run data seed
psql -d your_database -f migrations/002_system_seed.sql

# 3. Run optional features (if needed)
psql -d your_database -f migrations/003_log_system.sql
```

### Development Reset

```bash
# Drop and recreate database, then run migrations
dropdb your_database && createdb your_database
psql -d your_database -f migrations/001_system_schema.sql
psql -d your_database -f migrations/002_system_seed.sql
psql -d your_database -f migrations/003_log_system.sql
```

## File Organization

- **Core files** (`001_*.sql`): Essential system structure
- **Seed files** (`002_*.sql`): Initial data for core system
- **Feature files** (`003_*.sql` and above): Optional feature implementations

### Core System Features

The core system includes:

- User management (users, roles, user_roles)
- Menu management (menus, role_menus)
- Permission control system
- Performance views and triggers

### Optional Features

- **Log System**: Run `003_log_system.sql` if you need comprehensive logging
- **Future Features**: Additional feature-specific migrations can be added as needed

This separation allows you to:

- Run core system without optional features
- Maintain different feature sets for different environments
- Keep migrations focused and manageable

## Database Design

### Core Tables

1. **users** - User accounts

   - Basic user information: username, email, password, real_name
   - Soft delete: using `deleted_at` field

2. **roles** - User roles

   - Role information: role_name, description, status
   - Soft delete: using `deleted_at` field

3. **user_roles** - User-role associations

   - Many-to-many relationship: users can have multiple roles

4. **menus** - Menu and permission definitions

   - Menu information: title, path, component, icon
   - Hierarchical structure: via `parent_id`
   - Soft delete: using `deleted_at` field

5. **role_menus** - Role-menu associations
   - **Permission control core**: which menus roles can access
   - Many-to-many relationship: roles can access multiple menus

### Permission Model

Simple **Role-Based Menu Access Control**:

```
Users ←→ Roles ←→ Menus
```

- **Users** are associated with **Roles** via `user_roles`
- **Roles** are associated with **Menus** via `role_menus`
- **Permission control** = which menus users can see

### Soft Delete Strategy

- Uses `deleted_at` field for soft deletion
- `deleted_at IS NULL` means record is not deleted
- `deleted_at IS NOT NULL` means record is deleted
- Unique indexes include `WHERE deleted_at IS NULL` condition

## Database Configuration

### Environment Variables

```bash
# PostgreSQL connection configuration
DATABASE_URL=postgresql://username:password@localhost:5432/rustzen_admin
```

### Connection Pool Configuration

```toml
[database]
max_connections = 20
min_connections = 5
connect_timeout = 10
idle_timeout = 300
```

## 🚨 Important Notes

### Production Environment Deployment

1. **Backup Database**:

   ```bash
   pg_dump rustzen_admin > backup_$(date +%Y%m%d_%H%M%S).sql
   ```

2. **Check Permissions**: Ensure database user has sufficient permissions

3. **Test Migrations**: Test in development environment first

### Soft Delete Query

Query needs to filter deleted records:

```sql
-- Query active users
SELECT * FROM users WHERE deleted_at IS NULL;

-- Query active roles
SELECT * FROM roles WHERE deleted_at IS NULL;

-- Query active menus
SELECT * FROM menus WHERE deleted_at IS NULL;
```

## 🔍 Troubleshooting

### Common Issues

1. **Insufficient Permissions**: Check database user permissions
2. **Foreign Key Constraint Failure**: Check if associated data exists
3. **Unique Constraint Conflict**: Check data duplication (note soft delete conditions)

### Debug Commands

```bash
# Check migration status
sqlx migrate info --database-url $DATABASE_URL

# View table structure
psql -U username -d rustzen_admin -c "\d users"

# Check partition information (if log system is enabled)
SELECT * FROM get_log_partition_info();
```

## 📈 Extension Suggestions

If future needs are more complex, consider:

1. **Fine-grained Permissions**: Add operation permission table
2. **Data Permissions**: Add data range control
3. **Audit Logs**: Add `created_by`, `updated_by` fields
4. **Multi-tenant**: Add tenant isolation
5. **Cache Optimization**: Add Redis cache layer

## Default Super Admin Account

After running the seed file, you can login with:

- **Username**: `superadmin`
- **Password**: `rustzen@123`
- **Email**: `superadmin@example.com`

⚠️ **Important**: Change the default password after first login!

---

**Design Principle**: Start simple, then complex. Extend based on actual requirements.
