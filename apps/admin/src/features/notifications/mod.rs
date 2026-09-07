#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod accounting;
#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod admission;
#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod admission_repo;
#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod admission_types;
mod cursor;
mod handler;
#[allow(dead_code, reason = "started only by notification-selected Admin runtime")]
pub(crate) mod maintenance;
#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod pressure;
mod repo;
#[allow(dead_code, reason = "P5b internal admission is connected to producers in P6")]
mod retention;
mod service;
pub(crate) mod types;

use crate::infra::contract::{AccessPolicy, ContractRouter, OperationDescriptor};
use axum::routing::{get, post, put};
use sqlx::SqlitePool;

pub fn notification_routes() -> ContractRouter<SqlitePool> {
    ContractRouter::new()
        .get(
            "/",
            OperationDescriptor::ListNotifications,
            AccessPolicy::Authenticated,
            get(handler::list),
        )
        .expect("static notification contract")
        .get(
            "/unread-count",
            OperationDescriptor::GetNotificationUnreadCount,
            AccessPolicy::Authenticated,
            get(handler::unread_count),
        )
        .expect("static notification contract")
        .get(
            "/{id}",
            OperationDescriptor::GetNotification,
            AccessPolicy::Authenticated,
            get(handler::detail),
        )
        .expect("static notification contract")
        .put(
            "/{id}/read",
            OperationDescriptor::ReadNotification,
            AccessPolicy::Authenticated,
            put(handler::mark_read),
        )
        .expect("static notification contract")
        .post(
            "/read-all",
            OperationDescriptor::ReadAllNotifications,
            AccessPolicy::Authenticated,
            post(handler::mark_all_read),
        )
        .expect("static notification contract")
}

#[cfg(test)]
mod tests;
