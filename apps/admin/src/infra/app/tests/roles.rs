use super::*;

#[tokio::test]
async fn role_management_rejects_deletion_of_assigned_custom_role() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");
    activate_seed_owner(&pool).await;
    let menu_id: i64 = sqlx::query_scalar(
        "INSERT INTO menus (
             parent_id, name, code, menu_type, sort_order, status, is_system, is_manual,
             is_active
         ) VALUES (0, 'Role list', 'test:role:list', 2, 1, 1, FALSE, TRUE, TRUE)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("assignable permission menu");

    let (routes, _) = documented_protected_routes();
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .route_layer(middleware::from_fn_with_state((codec.clone(), TestLoader), auth_middleware))
        .with_state(pool.clone());
    let owner = session_token(&pool, &codec, 1, "owner").await;
    let create_response = app
        .clone()
        .oneshot(
            Request::post("/api/system/roles")
                .header("authorization", format!("Bearer {owner}"))
                .header("content-type", "application/json")
                .body(Body::from(
                    serde_json::json!({
                        "name": "Auditor",
                        "code": "auditor",
                        "status": 1,
                        "menuIds": [menu_id],
                        "description": "Can review role lists"
                    })
                    .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(create_response.status(), StatusCode::OK);

    let role_id: i64 = sqlx::query_scalar("SELECT id FROM roles WHERE code = 'auditor'")
        .fetch_one(&pool)
        .await
        .expect("created custom role");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(1_i64)
        .bind(role_id)
        .execute(&pool)
        .await
        .expect("assign custom role");

    let delete_response = app
        .oneshot(
            Request::delete(format!("/api/system/roles/{role_id}"))
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(delete_response.status(), StatusCode::BAD_REQUEST);
    let payload: serde_json::Value =
        serde_json::from_slice(&to_bytes(delete_response.into_body(), usize::MAX).await.unwrap())
            .expect("role deletion error");
    assert_eq!(payload["code"], 10002);
    assert!(payload["message"].as_str().unwrap().contains("assigned"));

    let role_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM roles WHERE id = ? AND deleted_at IS NULL")
            .bind(role_id)
            .fetch_one(&pool)
            .await
            .expect("role remains after refused deletion");
    let assignment_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM user_roles WHERE role_id = ?")
            .bind(role_id)
            .fetch_one(&pool)
            .await
            .expect("role assignment remains after refused deletion");
    assert_eq!(role_count, 1);
    assert_eq!(assignment_count, 1);
}

#[tokio::test]
async fn role_list_exposes_assignment_count_and_deletable_state() {
    let pool = SqlitePool::connect("sqlite::memory:").await.expect("pool");
    run_migrations(&pool).await.expect("migrations");

    let unassigned_role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status, is_system)
         VALUES ('Unassigned role', 'unassigned_role', 1, FALSE)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("unassigned role");
    let assigned_role_id: i64 = sqlx::query_scalar(
        "INSERT INTO roles (name, code, status, is_system)
         VALUES ('Assigned role', 'assigned_role', 1, FALSE)
         RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("assigned role");
    sqlx::query("INSERT INTO user_roles (user_id, role_id) VALUES (?, ?)")
        .bind(1_i64)
        .bind(assigned_role_id)
        .execute(&pool)
        .await
        .expect("assigned role relation");

    let (routes, _) = documented_protected_routes();
    let codec = JwtCodec::new("contract-test", 60);
    let app = Router::new()
        .merge(routes)
        .route_layer(middleware::from_fn_with_state((codec.clone(), TestLoader), auth_middleware))
        .with_state(pool);
    let owner = codec.encode(1, "owner").expect("owner token");
    let response = app
        .oneshot(
            Request::get("/api/system/roles?current=1&pageSize=20")
                .header("authorization", format!("Bearer {owner}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let payload: serde_json::Value =
        serde_json::from_slice(&to_bytes(response.into_body(), usize::MAX).await.unwrap())
            .expect("role list response");
    let rows = payload["data"].as_array().expect("role list data");
    let row_for = |code: &str| {
        rows.iter().find(|row| row["code"] == code).unwrap_or_else(|| panic!("missing role {code}"))
    };

    let unassigned = row_for("unassigned_role");
    assert_eq!(unassigned["id"], unassigned_role_id);
    assert_eq!(unassigned["assignedUserCount"], 0);
    assert_eq!(unassigned["deletable"], true);

    let assigned = row_for("assigned_role");
    assert_eq!(assigned["id"], assigned_role_id);
    assert_eq!(assigned["assignedUserCount"], 1);
    assert_eq!(assigned["deletable"], false);

    let owner = row_for("owner");
    assert_eq!(owner["deletable"], false);
}
