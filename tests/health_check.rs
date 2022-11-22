use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use tower::ServiceExt;
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::{new_router, run};

struct TestApp {
    pub db_pool: PgPool,
}

impl TestApp {
    pub async fn new() -> Self {
        let mut configuration = get_configuration().expect("Failed to get configuration");
        configuration.database.database_name = Uuid::new_v4().to_string();

        let db_pool = configure_database(&configuration.database).await;

        TestApp { db_pool }
    }
}

async fn configure_database(settings: &DatabaseSettings) -> PgPool {
    let connection = PgPoolOptions::new()
        .connect(&settings.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .await
        .expect("Failed to create database.");

    // Migrate database
    // let connection_pool = PgPool::connect(&settings.connection_string())
    //     .await
    //     .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection)
        .await
        .expect("Failed to migrate the database");
    connection
}

#[tokio::test]
async fn health_check_works() {
    let app = TestApp::new().await;

    let response = new_router(app.db_pool.clone())
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .expect("Failed to create request");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    assert_eq!(&body[..], b"OK");
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let app = TestApp::new().await;

    let body = Body::from("name=le%20guin&email=ursula_le_guin%40gmail.com");

    let response = new_router(app.db_pool.clone())
        .oneshot(
            Request::builder()
                .uri("/subscriptions")
                .header("Content-Type", "application/x-www-form-urlencoded")
                .method(Method::POST)
                .body(body)
                .expect("Failed to create request"),
        )
        .await
        .expect("Failed to call api");

    assert_eq!(response.status(), StatusCode::OK);

    let saved = sqlx::query!("SELECT email, name from subscriptions")
        .fetch_one(&app.db_pool.clone())
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let app = TestApp::new().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];

    for (invalid_body, error_message) in test_cases {
        let body = Body::from(invalid_body);

        let response = new_router(app.db_pool.clone())
            .oneshot(
                Request::builder()
                    .uri("/subscriptions")
                    .method(Method::POST)
                    .header("Content-Type", "application/x-www-form-urlencoded")
                    .body(body)
                    .expect("Failed to create request"),
            )
            .await
            .expect("Failed to call api");

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        // assert_eq!(response.body(), error_message);
    }
}
