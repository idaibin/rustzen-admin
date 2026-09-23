use super::*;

pub(super) fn build_menu_seed_records(raw_codes: &[String]) -> Vec<MenuSeedRecord> {
    expand_permission_codes(raw_codes)
        .into_iter()
        .filter(|code| code != SYSTEM_WILDCARD)
        .map(|permission_code| menu_seed_record(&permission_code))
        .collect()
}

pub(super) fn expand_permission_codes(raw_codes: &[String]) -> Vec<String> {
    let mut expanded = BTreeSet::new();

    for raw_code in raw_codes {
        expand_permission_code(raw_code, &mut expanded);
    }

    expanded.into_iter().collect()
}

fn expand_permission_code(raw_code: &str, expanded: &mut BTreeSet<String>) {
    let mut current = raw_code.trim().to_string();
    if current.is_empty() {
        return;
    }

    loop {
        expanded.insert(current.clone());
        match parent_permission_code(&current) {
            Some(parent_code) => current = parent_code,
            None => break,
        }
    }
}

fn parent_permission_code(permission_code: &str) -> Option<String> {
    if permission_code == SYSTEM_WILDCARD {
        return None;
    }

    let segments: Vec<&str> = permission_code.split(':').collect();
    if segments.len() <= 1 {
        return None;
    }

    if segments.last() == Some(&"*") {
        if segments.len() <= 2 {
            None
        } else {
            Some(format!("{}:*", segments[..segments.len() - 2].join(":")))
        }
    } else {
        Some(format!("{}:*", segments[..segments.len() - 1].join(":")))
    }
}

pub(super) fn menu_seed_record(permission_code: &str) -> MenuSeedRecord {
    MenuSeedRecord {
        permission_code: permission_code.to_string(),
        parent_code: parent_permission_code(permission_code),
        title: permission_title(permission_code),
        path: None,
        icon: None,
        sort_order: 0,
        status: MENU_STATUS_VISIBLE,
        menu_type: menu_type(permission_code),
        is_system: true,
        is_manual: false,
        module_id: None,
        module_menu_code: None,
        is_active: true,
    }
}

fn menu_type(permission_code: &str) -> i16 {
    if permission_code.ends_with(":*") {
        MENU_TYPE_DIRECTORY
    } else if permission_code.ends_with(":list") || permission_code.ends_with(":view") {
        MENU_TYPE_MENU
    } else {
        MENU_TYPE_BUTTON
    }
}

pub(super) fn permission_title(permission_code: &str) -> String {
    if permission_code == SYSTEM_WILDCARD {
        return "全部权限".to_string();
    }

    let segments: Vec<&str> = permission_code.split(':').collect();
    if segments.is_empty() {
        return String::new();
    }

    if permission_code.ends_with(":*") {
        let meaningful_segments = &segments[..segments.len().saturating_sub(1)];
        let base = localize_segments(meaningful_segments);
        if base.is_empty() || base.ends_with("管理") { base } else { format!("{base}管理") }
    } else {
        localize_segments(&segments)
    }
}

fn localize_segment(segment: &str) -> String {
    match segment {
        "dashboard" => "仪表盘",
        "system" => "系统",
        "user" => "用户",
        "role" => "角色",
        "menu" => "菜单",
        "module" => "模块",
        "status" => "状态",
        "manage" => "管理",
        "log" => "日志",
        "task" => "任务",
        "deploy" => "部署",
        "monitor" => "监控",
        "node" => "节点",
        "check" => "服务监控",
        "incident" => "事件",
        "settings" => "设置",
        "insights" => "分析",
        "overview" => "概览",
        "project" => "项目",
        "event" => "事件",
        "page" => "页面",
        "api" => "API",
        "report" | "reports" => "报表",
        "flow" => "流程",
        "run" => "执行",
        "schedule" => "计划",
        "template" => "模板",
        "create" => "新增",
        "delete" => "删除",
        "list" => "列表",
        "options" => "选项",
        "update" => "修改",
        "password" => "密码",
        "view" => "查看",
        "export" => "导出",
        "analyze" => "分析",
        "recover" => "恢复",
        "restart" => "重启",
        other => return humanize_segment(other),
    }
    .to_string()
}

fn humanize_segment(segment: &str) -> String {
    segment
        .split(['_', '-', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str().to_lowercase()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn localize_segments(segments: &[&str]) -> String {
    segments.iter().map(|segment| localize_segment(segment)).collect::<Vec<_>>().join("")
}

pub(super) async fn retire_stale_core_permissions(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    current_records: &[MenuSeedRecord],
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    let mut query = sqlx::QueryBuilder::<Sqlite>::new(
        "UPDATE menus
         SET is_active = FALSE, updated_at = ",
    );
    query
        .push_bind(now)
        .push(
            " WHERE module_id IS NULL
               AND is_system = TRUE
               AND code <> ",
        )
        .push_bind(SYSTEM_WILDCARD)
        .push(" AND deleted_at IS NULL AND code NOT IN (");
    let mut codes = query.separated(", ");
    for record in current_records {
        codes.push_bind(&record.permission_code);
    }
    codes.push_unseparated(")");
    query
        .build()
        .execute(&mut **tx)
        .await
        .map_err(database_error("retiring stale core permissions"))?;

    Ok(())
}

pub(super) async fn upsert_menu_seed_record(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    record: &MenuSeedRecord,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "INSERT INTO menus (
             parent_id, parent_code, name, code, menu_type, status, is_system, is_manual,
             path, icon, sort_order, module_id, module_menu_code, is_active, created_at, updated_at
         )
         VALUES (0, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON CONFLICT(code) WHERE deleted_at IS NULL DO UPDATE
         SET parent_code = EXCLUDED.parent_code,
             name = EXCLUDED.name,
             menu_type = EXCLUDED.menu_type,
             status = EXCLUDED.status,
             is_system = EXCLUDED.is_system,
             is_manual = EXCLUDED.is_manual,
             path = EXCLUDED.path,
             icon = EXCLUDED.icon,
             sort_order = EXCLUDED.sort_order,
             module_id = EXCLUDED.module_id,
             module_menu_code = EXCLUDED.module_menu_code,
             is_active = EXCLUDED.is_active,
             updated_at = EXCLUDED.updated_at
         WHERE menus.is_manual = FALSE",
    )
    .bind(record.parent_code.as_deref())
    .bind(&record.title)
    .bind(&record.permission_code)
    .bind(record.menu_type)
    .bind(record.status)
    .bind(record.is_system)
    .bind(record.is_manual)
    .bind(record.path.as_deref())
    .bind(record.icon.as_deref())
    .bind(record.sort_order)
    .bind(record.module_id.as_deref())
    .bind(record.module_menu_code.as_deref())
    .bind(record.is_active)
    .bind(now)
    .bind(now)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(
            "Database error upserting permission seed {}: {:?}",
            record.permission_code,
            e
        );
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}

pub(super) async fn refresh_menu_parent_id(
    tx: &mut sqlx::Transaction<'_, Sqlite>,
    permission_code: &str,
) -> Result<(), ServiceError> {
    let now = Utc::now().naive_utc();
    sqlx::query(
        "UPDATE menus
         SET parent_id = COALESCE(
             (SELECT parent.id FROM menus parent WHERE parent.code = menus.parent_code AND parent.deleted_at IS NULL),
             0
         ),
         updated_at = ?
         WHERE code = ? AND deleted_at IS NULL AND is_manual = FALSE",
    )
    .bind(now)
    .bind(permission_code)
    .execute(&mut **tx)
    .await
    .map_err(|e| {
        tracing::error!(
            "Database error refreshing parent_id for permission {}: {:?}",
            permission_code,
            e
        );
        ServiceError::DatabaseQueryFailed
    })?;

    Ok(())
}
