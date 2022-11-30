use crate::domain::{NewSubscriber, SubscriberEmail, SubscriberName};
use crate::error::AppError;

use super::SubscriptionFormData;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Form};
use chrono::Utc;
use sqlx::PgPool;
use tracing::Instrument;
use tracing::{debug, error, info, info_span};
use uuid::Uuid;

impl TryFrom<SubscriptionFormData> for NewSubscriber {
    type Error = AppError;

    fn try_from(form: SubscriptionFormData) -> Result<Self, Self::Error> {
        let name = match SubscriberName::parse(form.name) {
            Ok(name) => name,
            Err(e) => return Err(AppError::BadRequest(e.to_string())),
        };
        let email = match SubscriberEmail::parse(form.email) {
            Ok(email) => email,
            Err(e) => return Err(AppError::BadRequest(e.to_string())),
        };

        Ok(NewSubscriber { email, name })
    }
}

#[axum_macros::debug_handler]
#[tracing::instrument(name = "Adding a new subscriber", skip(form, db_connection), fields(
    subscriber_email = %form.email,
    subscriber_name = %form.name
))]
pub async fn subscriptions(
    State(db_connection): State<PgPool>,
    Form(form): Form<SubscriptionFormData>,
) -> Result<String, AppError> {
    let subscriber = match form.try_into() {
        Ok(subscriber) => subscriber,
        Err(e) => return Err(e),
    };

    match insert_subscriber(&db_connection, &subscriber).await {
        Ok(_) => {
            info!("New subscriber details has been saved");
            return Ok("New subscriber details has been saved".to_owned());
        }
        Err(err) => {
            error!("Failed to save subscriber details {:?}", err);
            return Err(AppError::InternalServerError(format!(
                "Failed to save subscriber details {:?}",
                err
            )));
        }
    }
}

#[tracing::instrument(
    name = "Insert a new subscriber into database",
    skip(subscriber, db_connection)
)]
pub async fn insert_subscriber(
    db_connection: &PgPool,
    subscriber: &NewSubscriber,
) -> anyhow::Result<()> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscribed_at)
        VALUES ($1, $2, $3, $4)"#,
        Uuid::new_v4(),
        subscriber.email.as_ref(),
        subscriber.name.as_ref(),
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
