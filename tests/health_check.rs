use axum::body::Body;
use axum::http::{Request, StatusCode};
use std::net::TcpListener;
use tower::ServiceExt;
use zero2prod::{new_router, run};

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let port = addr.port();

    let server = run(addr);
    let _ = tokio::spawn(async move { server.await.expect("Failed to start HTTP server") });

    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    let response = new_router()
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
