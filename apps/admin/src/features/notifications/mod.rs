mod cursor;
mod handler;
mod repo;
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
