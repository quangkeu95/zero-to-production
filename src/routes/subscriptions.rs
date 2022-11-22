use super::SubscriptionFormData;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use chrono::Utc;
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

pub async fn subscriptions(
    Form(form): Form<SubscriptionFormData>,
    db_connection: Extension<PgPool>,
) -> impl IntoResponse {
    let _ = sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&*db_connection)
    .await;
    StatusCode::OK
}
