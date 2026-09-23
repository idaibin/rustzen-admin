use chrono::Utc;
use rustzen_auth::auth::{AuthClaims, JwtCodec};
use sqlx::{SqlitePool, sqlite::SqlitePoolOptions};

use crate::features::{
    auth::session::SessionRepository,
    system::menu::{repo::MenuRepository, types::UpdateMenuPayload},
};

async fn revoked_owner() -> (SqlitePool, AuthClaims) {
    let pool =
        SqlitePoolOptions::new().max_connections(1).connect("sqlite::memory:").await.expect("pool");
    crate::infra::db::run_migrations(&pool).await.expect("migrations");
    sqlx::query("UPDATE users SET status=1 WHERE id=1").execute(&pool).await.expect("enable owner");
    let epoch: i64 = sqlx::query_scalar("SELECT auth_epoch FROM users WHERE id=1")
        .fetch_one(&pool)
        .await
        .expect("epoch");
    let now = Utc::now().timestamp();
    let sid = uuid::Uuid::new_v4().to_string();
    let claims = JwtCodec::new("test", 3600).claims_at(1, "owner", &sid, epoch, now);
    SessionRepository::create(&pool, 1, &sid, epoch, now, claims.exp as i64)
        .await
        .expect("session");
    SessionRepository::revoke_sid(&pool, 1, &sid, now).await.expect("revoke");
    (pool, claims)
}

#[tokio::test]
async fn revoked_actor_cannot_update_or_delete_menu_rows() {
    let (pool, actor) = revoked_owner().await;
    let navigation_id: i64 = sqlx::query_scalar(
        "INSERT INTO module_navigation
         (name,code,status,is_manual,sort_order,path,icon,module_id,module_menu_code)
         VALUES ('Before','monitor:view',1,TRUE,1,'/monitoring','monitor','monitor','monitor')
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("navigation");
    let menu_id: i64 = sqlx::query_scalar(
        "INSERT INTO menus
         (name,code,menu_type,status,is_system,is_manual,sort_order)
         VALUES ('Manual','manual:test',2,1,FALSE,TRUE,1) RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("menu");

    assert!(
        MenuRepository::update_navigation_authorized(
            &pool,
            navigation_id,
            &UpdateMenuPayload { name: "After".into(), icon: None, sort_order: 2, status: 2 },
            &actor,
        )
        .await
        .is_err()
    );
    assert!(MenuRepository::disable_authorized(&pool, menu_id, &actor).await.is_err());
    assert_eq!(
        sqlx::query_scalar::<_, String>("SELECT name FROM module_navigation WHERE id=?")
            .bind(navigation_id)
            .fetch_one(&pool)
            .await
            .expect("navigation name"),
        "Before"
    );
    assert_eq!(
        sqlx::query_scalar::<_, i16>("SELECT status FROM menus WHERE id=?")
            .bind(menu_id)
            .fetch_one(&pool)
            .await
            .expect("menu status"),
        1
    );
}

#[cfg(feature = "full")]
#[tokio::test]
async fn revoked_actor_cannot_change_module_enabled_state() {
    let (pool, actor) = revoked_owner().await;
    let before: bool = sqlx::query_scalar("SELECT enabled FROM modules WHERE id='monitor'")
        .fetch_one(&pool)
        .await
        .expect("module");
    assert!(
        crate::features::modules::repo::ModuleRepository::set_enabled_authorized(
            &pool, "monitor", !before, &actor,
        )
        .await
        .is_err()
    );
    assert_eq!(
        sqlx::query_scalar::<_, bool>("SELECT enabled FROM modules WHERE id='monitor'")
            .fetch_one(&pool)
            .await
            .expect("module after"),
        before
    );
}
