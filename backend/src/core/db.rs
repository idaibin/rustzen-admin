use sqlx::{PgPool, postgres::PgPoolOptions};
use std::time::Duration;

/// 数据库连接池配置
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub idle_timeout: Duration,
}

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

/// 创建数据库连接池
pub async fn create_pool(config: DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.connect_timeout)
        .idle_timeout(config.idle_timeout)
        .connect(&config.url)
        .await
}

/// 创建默认配置的数据库连接池
pub async fn create_default_pool() -> Result<PgPool, sqlx::Error> {
    let config = DatabaseConfig::default();
    create_pool(config).await
}

/// 测试数据库连接
pub async fn test_connection(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;

    tracing::info!("数据库连接测试成功");
    Ok(())
}
