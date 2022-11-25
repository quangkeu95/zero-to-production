use super::SubscriptionFormData;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use tracing::{debug, error, info, info_span};
use uuid::Uuid;

#[tracing::instrument(name = "Adding a new subscriber", skip(form, db_connection), fields(
    request_id = %Uuid::new_v4(),
    subscriber_email = %form.email,
    subscriber_name = %form.name
))]
pub async fn subscriptions(
    Form(form): Form<SubscriptionFormData>,
    db_connection: Extension<PgPool>,
) -> impl IntoResponse {
    info!("Saving new subscriber details in the database");

    match insert_subscriber(&db_connection, &form).await {
        Ok(_) => {
            info!("New subscriber details has been saved");
            return StatusCode::OK;
        }
        Err(err) => {
            error!("Failed to save subscriber details {:?}", err);
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }
}

#[tracing::instrument(
    name = "Insert a new subscriber into database",
    skip(form, db_connection)
)]
pub async fn insert_subscriber(
    db_connection: &PgPool,
    form: &SubscriptionFormData,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(&*db_connection)
    .await
    .map_err(|e| {
        error!("Failed to execute query {:?}", e);
        e
    })?;
    Ok(())
}
