use super::*;

pub(super) async fn reconcile_navigation(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    manifest: &ModuleManifest,
) -> Result<(), ServiceError> {
    sqlx::query("UPDATE module_navigation SET is_active=FALSE WHERE module_id=?")
        .bind(&manifest.module)
        .execute(&mut **tx)
        .await
        .map_err(database_error("retiring module navigation"))?;
    for menu in &manifest.menus {
        sqlx::query(
            "INSERT INTO module_navigation
             (module_id,module_menu_code,code,name,path,icon,sort_order)
             VALUES(?,?,?,?,?,?,?)
             ON CONFLICT(module_id,module_menu_code) DO UPDATE SET
               code=excluded.code,path=excluded.path,is_active=TRUE,
               name=CASE WHEN module_navigation.is_manual THEN module_navigation.name ELSE excluded.name END,
               icon=CASE WHEN module_navigation.is_manual THEN module_navigation.icon ELSE excluded.icon END,
               sort_order=CASE WHEN module_navigation.is_manual THEN module_navigation.sort_order ELSE excluded.sort_order END,
               updated_at=CURRENT_TIMESTAMP"
        ).bind(&manifest.module).bind(&menu.code).bind(&menu.permission)
            .bind(&menu.title).bind(&menu.path).bind(&menu.icon).bind(menu.sort_order)
            .execute(&mut **tx).await.map_err(database_error("reconciling module navigation"))?;
    }
    Ok(())
}
