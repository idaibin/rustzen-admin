use sqlx::{QueryBuilder, Sqlite, SqlitePool};

use crate::common::error::ServiceError;

use super::types::{
    DeployComponent, DeploymentItem, DeploymentPayload, DeploymentRow, ListDeploymentsQuery,
};

pub struct DeployRepository {
    pool: SqlitePool,
}

impl DeployRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list(
        &self,
        query: &ListDeploymentsQuery,
        offset: i64,
        limit: i64,
    ) -> Result<(Vec<DeploymentItem>, i64), ServiceError> {
        let total = self.count(query).await?;
        if total == 0 {
            return Ok((Vec::new(), 0));
        }

        let mut sql = base_select();
        push_filters(&mut sql, query);
        sql.push(" ORDER BY created_at DESC, id DESC LIMIT ")
            .push_bind(limit)
            .push(" OFFSET ")
            .push_bind(offset);

        let rows = sql
            .build_query_as::<DeploymentRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;
        let items = rows
            .into_iter()
            .map(row_to_item)
            .collect::<Result<Vec<_>, _>>()?;
        Ok((items, total))
    }

    async fn count(&self, query: &ListDeploymentsQuery) -> Result<i64, ServiceError> {
        let mut sql = QueryBuilder::new("SELECT COUNT(*) FROM deploy_versions WHERE deleted_at IS NULL");
        push_filters(&mut sql, query);
        let (total,): (i64,) = sql
            .build_query_as()
            .fetch_one(&self.pool)
            .await
            .map_err(map_db_error)?;
        Ok(total)
    }

    pub async fn find_by_id(&self, id: i64) -> Result<DeploymentItem, ServiceError> {
        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT id, component, version, arch, file_path, file_size, file_hash,
                   is_current, is_deployed, is_expired, deployed_at, expired_at, deleted_at,
                   deployed_by, notes, created_at, updated_at
            FROM deploy_versions
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?
        .ok_or_else(|| ServiceError::NotFound("Deploy version".to_string()))?;
        row_to_item(row)
    }

    pub async fn version_exists(
        &self,
        component: &DeployComponent,
        version: &str,
        arch: &str,
    ) -> Result<bool, ServiceError> {
        let count = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM deploy_versions WHERE component = ? AND version = ? AND arch = ?",
        )
        .bind(component_to_str(component))
        .bind(version)
        .bind(arch)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(count > 0)
    }

    pub async fn insert(&self, payload: &DeploymentPayload) -> Result<DeploymentItem, ServiceError> {
        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            INSERT INTO deploy_versions (
                component, version, arch, file_path, file_size, file_hash,
                notes, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
            RETURNING id, component, version, arch, file_path, file_size, file_hash,
                      is_current, is_deployed, is_expired, deployed_at, expired_at, deleted_at,
                      deployed_by, notes, created_at, updated_at
            "#,
        )
        .bind(component_to_str(&payload.component))
        .bind(&payload.version)
        .bind(&payload.arch)
        .bind(&payload.file_path)
        .bind(payload.file_size)
        .bind(&payload.file_hash)
        .bind(payload.notes.as_deref())
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;
        row_to_item(row)
    }

    pub async fn find_current(
        &self,
        component: &DeployComponent,
        arch: &str,
    ) -> Result<Option<DeploymentItem>, ServiceError> {
        let row = sqlx::query_as::<_, DeploymentRow>(
            r#"
            SELECT id, component, version, arch, file_path, file_size, file_hash,
                   is_current, is_deployed, is_expired, deployed_at, expired_at, deleted_at,
                   deployed_by, notes, created_at, updated_at
            FROM deploy_versions
            WHERE component = ? AND arch = ? AND is_current = 1 AND deleted_at IS NULL
            ORDER BY COALESCE(deployed_at, created_at) DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(component_to_str(component))
        .bind(arch)
        .fetch_optional(&self.pool)
        .await
        .map_err(map_db_error)?;
        row.map(row_to_item).transpose()
    }

    pub async fn set_current(
        &self,
        component: &DeployComponent,
        arch: &str,
        version_id: i64,
        deployed_by: Option<&str>,
    ) -> Result<(), ServiceError> {
        let mut tx = self.pool.begin().await.map_err(map_db_error)?;

        sqlx::query(
            r#"
            UPDATE deploy_versions
            SET is_current = 0, updated_at = CURRENT_TIMESTAMP
            WHERE component = ? AND arch = ? AND is_current = 1
            "#,
        )
        .bind(component_to_str(component))
        .bind(arch)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;

        let result = sqlx::query(
            r#"
            UPDATE deploy_versions
            SET is_current = 1,
                is_deployed = 1,
                deployed_at = CURRENT_TIMESTAMP,
                deployed_by = ?,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(deployed_by)
        .bind(version_id)
        .execute(&mut *tx)
        .await
        .map_err(map_db_error)?;
        if result.rows_affected() == 0 {
            return Err(ServiceError::NotFound("Deploy version".to_string()));
        }

        tx.commit().await.map_err(map_db_error)?;

        if let Err(err) = self.expire_old_versions(component, arch).await {
            tracing::warn!(
                "Failed to expire old deploy versions for component={} arch={}: {}",
                component_to_str(component),
                arch,
                err
            );
        }

        Ok(())
    }

    pub async fn mark_expired(
        &self,
        version_id: i64,
        notes: Option<&str>,
    ) -> Result<DeploymentItem, ServiceError> {
        let result = sqlx::query(
            r#"
            UPDATE deploy_versions
            SET is_expired = 1,
                expired_at = CURRENT_TIMESTAMP,
                notes = COALESCE(?, notes),
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND is_current = 0 AND deleted_at IS NULL
            "#,
        )
        .bind(notes)
        .bind(version_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        if result.rows_affected() == 0 {
            self.ensure_not_current(version_id, "expire").await?;
        }

        self.find_by_id(version_id).await
    }

    pub async fn delete_by_id(&self, version_id: i64) -> Result<DeploymentItem, ServiceError> {
        let version = self.find_by_id(version_id).await?;
        if version.is_current {
            return Err(ServiceError::InvalidOperation(
                "Cannot delete the current deploy version".to_string(),
            ));
        }

        sqlx::query(
            r#"
            UPDATE deploy_versions
            SET deleted_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE id = ? AND deleted_at IS NULL
            "#,
        )
        .bind(version_id)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;

        Ok(version)
    }

    pub async fn expired_non_current(
        &self,
        component: Option<&DeployComponent>,
    ) -> Result<Vec<DeploymentItem>, ServiceError> {
        let query = ListDeploymentsQuery {
            current: None,
            page_size: None,
            component: component.cloned(),
            is_current: Some(false),
            is_deployed: None,
            is_expired: Some(true),
            search: None,
        };

        let mut sql = base_select();
        push_filters(&mut sql, &query);
        sql.push(" ORDER BY created_at DESC, id DESC");
        let rows = sql
            .build_query_as::<DeploymentRow>()
            .fetch_all(&self.pool)
            .await
            .map_err(map_db_error)?;
        rows.into_iter().map(row_to_item).collect()
    }

    async fn expire_old_versions(
        &self,
        component: &DeployComponent,
        arch: &str,
    ) -> Result<u64, ServiceError> {
        let result = sqlx::query(
            r#"
            UPDATE deploy_versions
            SET is_expired = 1,
                expired_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE component = ?
              AND arch = ?
              AND is_deployed = 1
              AND is_expired = 0
              AND deleted_at IS NULL
              AND id NOT IN (
                SELECT id FROM deploy_versions
                WHERE component = ?
                  AND arch = ?
                  AND is_deployed = 1
                  AND is_expired = 0
                  AND deleted_at IS NULL
                ORDER BY COALESCE(deployed_at, created_at) DESC, id DESC
                LIMIT 3
              )
            "#,
        )
        .bind(component_to_str(component))
        .bind(arch)
        .bind(component_to_str(component))
        .bind(arch)
        .execute(&self.pool)
        .await
        .map_err(map_db_error)?;
        Ok(result.rows_affected())
    }

    async fn ensure_not_current(&self, version_id: i64, action: &str) -> Result<(), ServiceError> {
        let is_current = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM deploy_versions WHERE id = ? AND is_current = 1 AND deleted_at IS NULL",
        )
        .bind(version_id)
        .fetch_one(&self.pool)
        .await
        .map_err(map_db_error)?;

        if is_current > 0 {
            return Err(ServiceError::InvalidOperation(format!(
                "Cannot {action} the current deploy version"
            )));
        }

        Err(ServiceError::NotFound("Deploy version".to_string()))
    }
}

fn base_select() -> QueryBuilder<Sqlite> {
    QueryBuilder::new(
        r#"
        SELECT id, component, version, arch, file_path, file_size, file_hash,
               is_current, is_deployed, is_expired, deployed_at, expired_at, deleted_at,
               deployed_by, notes, created_at, updated_at
        FROM deploy_versions
        WHERE deleted_at IS NULL
        "#,
    )
}

fn push_filters(query: &mut QueryBuilder<Sqlite>, params: &ListDeploymentsQuery) {
    if let Some(component) = &params.component {
        query.push(" AND component = ").push_bind(component_to_str(component));
    }
    if let Some(is_current) = params.is_current {
        query.push(" AND is_current = ").push_bind(bool_to_i64(is_current));
    }
    if let Some(is_deployed) = params.is_deployed {
        query.push(" AND is_deployed = ").push_bind(bool_to_i64(is_deployed));
    }
    if let Some(is_expired) = params.is_expired {
        query.push(" AND is_expired = ").push_bind(bool_to_i64(is_expired));
    }

    let Some(search) = params.search.as_deref().map(str::trim).filter(|value| !value.is_empty()) else {
        return;
    };
    let pattern = format!("%{}%", search.to_lowercase());
    query
        .push(" AND (LOWER(version) LIKE ")
        .push_bind(pattern.clone())
        .push(" OR LOWER(file_hash) LIKE ")
        .push_bind(pattern.clone())
        .push(" OR LOWER(notes) LIKE ")
        .push_bind(pattern)
        .push(")");
}

fn row_to_item(row: DeploymentRow) -> Result<DeploymentItem, ServiceError> {
    Ok(DeploymentItem {
        id: row.id,
        component: component_from_str(&row.component)?,
        version: row.version,
        arch: row.arch,
        file_path: row.file_path,
        file_size: row.file_size,
        file_hash: row.file_hash,
        is_current: row.is_current,
        is_deployed: row.is_deployed,
        is_expired: row.is_expired,
        deployed_at: row.deployed_at,
        expired_at: row.expired_at,
        deleted_at: row.deleted_at,
        deployed_by: row.deployed_by,
        notes: row.notes,
        created_at: row.created_at,
        updated_at: row.updated_at,
    })
}

pub fn component_to_str(component: &DeployComponent) -> &'static str {
    match component {
        DeployComponent::Server => "server",
        DeployComponent::Web => "web",
    }
}

fn component_from_str(raw: &str) -> Result<DeployComponent, ServiceError> {
    match raw {
        "server" => Ok(DeployComponent::Server),
        "web" => Ok(DeployComponent::Web),
        other => Err(ServiceError::InvalidOperation(format!(
            "Invalid deploy component: {other}"
        ))),
    }
}

fn bool_to_i64(value: bool) -> i64 {
    if value { 1 } else { 0 }
}

fn map_db_error(err: sqlx::Error) -> ServiceError {
    tracing::error!("Deploy database error: {:?}", err);
    ServiceError::DatabaseQueryFailed
}
