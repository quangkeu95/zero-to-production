use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Method, Request, StatusCode};
use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool};
use std::net::TcpListener;
use std::str::FromStr;
use std::time::Duration;
use test_context::{test_context, AsyncTestContext};
use tower::ServiceExt;
use tracing::log::LevelFilter;
use tracing::{debug, error, info};
use uuid::Uuid;
use zero2prod::configuration::{get_configuration, DatabaseSettings};
use zero2prod::new_router;
use zero2prod::telemetry;

static TRACING: Lazy<()> = Lazy::new(|| {
    let test_log_level = match std::env::var("TEST_LOG_LEVEL") {
        Ok(log_level) => match LevelFilter::from_str(log_level.as_str()) {
            Ok(filter_level) => filter_level,
            Err(_) => LevelFilter::Off,
        },
        Err(_) => LevelFilter::Off,
    };

    let tracing_options = telemetry::TracingOptionsBuilder::default()
        .crate_level(test_log_level)
        .tower_http_level(LevelFilter::Off)
        .build()
        .unwrap();
    telemetry::init_tracing("zero2prod".into(), tracing_options);
});

struct TestApp {
    pub db_name: String,
    pub db_pool: PgPool,
    pub settings: DatabaseSettings,
}

impl TestApp {
    // pub async fn new() -> Self {
    //     Lazy::force(&TRACING);

    //     let mut configuration = get_configuration().expect("Failed to get configuration");
    //     configuration.database.database_name = Uuid::new_v4().to_string();

    //     let db_pool = configure_database(&configuration.database).await;

    //     TestApp {
    //         db_name: configuration.database.database_name.clone(),
    //         db_pool,
    //     }
    // }

    pub async fn drop_db(&self) -> anyhow::Result<()> {
        // need to drop current connection to database first
        self.db_pool.close().await;

        let connection = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&self.settings.connection_string_without_db().expose_secret())
            .await
            .expect("Failed to connect to Postgres");
        let _ = connection
            .execute(format!(r#"DROP DATABASE "{}";"#, self.db_name).as_str())
            .await
            .expect("Failed to drop database");
        Ok(())
    }
}

#[async_trait]
impl AsyncTestContext for TestApp {
    async fn setup() -> TestApp {
        Lazy::force(&TRACING);

        let mut configuration = get_configuration().expect("Failed to get configuration");
        configuration.database.database_name = Uuid::new_v4().to_string();

        let db_pool = configure_database(&configuration.database).await;

        TestApp {
            db_name: configuration.database.database_name.clone(),
            db_pool,
            settings: configuration.database.clone(),
        }
    }

    async fn teardown(self) {
        match self.drop_db().await {
            Ok(_) => println!("finished drop database {:?}", self.db_name),
            Err(err) => println!("error drop database {:?}", err),
        }
    }
}

async fn configure_database(settings: &DatabaseSettings) -> PgPool {
    let connection = PgPoolOptions::new()
        .connect(&settings.connection_string_without_db().expose_secret())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, settings.database_name).as_str())
        .await
        .expect("Failed to create database.");

    println!("database {:?} created!", settings.database_name);

    let return_connection = PgPoolOptions::new()
        .max_connections(10)
        .connect(&settings.connection_string().expose_secret())
        .await
        .expect("Failed to connect to Postgres");

    // Migrate database
    sqlx::migrate!("./migrations")
        .run(&return_connection)
        .await
        .expect("Failed to migrate the database");

    return_connection
}

#[test_context(TestApp)]
#[tokio::test]
async fn health_check_works(app: &mut TestApp) {
    // let app = TestApp::new().await;

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

#[test_context(TestApp)]
#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data(app: &mut TestApp) {
    // let app = TestApp::new().await;

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

#[test_context(TestApp)]
#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing(app: &mut TestApp) {
    // let app = TestApp::new().await;

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
