use axum::{
    extract::Request,
    http::{HeaderName, HeaderValue, StatusCode},
    middleware::{self, Next},
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse, Response,
    },
    routing::{get, post},
    Json, Router,
};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use std::{convert::Infallible, time::Duration};
use tokio::time::timeout;

#[derive(Debug, Deserialize)]
pub struct ChatRequest {
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: &'static str,
}

pub fn app() -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/chat", post(chat))
        .route("/stream", get(streaming))
        .layer(middleware::from_fn(request_id))
}

async fn health() -> StatusCode {
    StatusCode::OK
}

async fn request_id(req: Request, next: Next) -> Response {
    let request_id = req
        .headers()
        .get("x-request-id")
        .cloned()
        .unwrap_or_else(|| HeaderValue::from_static("generated-demo-id"));

    let mut response = next.run(req).await;
    response
        .headers_mut()
        .insert(HeaderName::from_static("x-request-id"), request_id);
    response
}

async fn chat(Json(input): Json<ChatRequest>) -> Response {
    if input.message.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorBody {
                code: "invalid_request",
                message: "message is required",
            }),
        )
            .into_response();
    }

    match timeout(
        Duration::from_millis(50),
        simulated_provider_call(input.message),
    )
    .await
    {
        Ok(Ok(text)) => (StatusCode::OK, text).into_response(),
        Ok(Err(_)) => (
            StatusCode::BAD_GATEWAY,
            Json(ErrorBody {
                code: "provider_error",
                message: "upstream provider failed",
            }),
        )
            .into_response(),
        Err(_) => (
            StatusCode::GATEWAY_TIMEOUT,
            Json(ErrorBody {
                code: "provider_timeout",
                message: "upstream provider timed out",
            }),
        )
            .into_response(),
    }
}

async fn simulated_provider_call(message: String) -> Result<String, ()> {
    if message == "fail" {
        return Err(());
    }
    Ok(format!("echo: {message}"))
}

async fn streaming() -> Sse<impl futures_util::Stream<Item = Result<Event, Infallible>>> {
    let events = vec![
        Ok(Event::default().event("delta").data("first")),
        Ok(Event::default().event("delta").data("second")),
        Ok(Event::default().event("done").data("{}")),
    ];
    Sse::new(stream::iter(events)).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::Body,
        http::{Method, Request},
    };
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_is_ok_and_has_request_id() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/healthz")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response.headers().contains_key("x-request-id"));
    }

    #[tokio::test]
    async fn empty_message_is_typed_bad_request() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":""}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body.contains("invalid_request"));
    }

    #[tokio::test]
    async fn provider_failure_is_not_reported_as_success() {
        let response = app()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/chat")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"message":"fail"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    }

    #[tokio::test]
    async fn streaming_endpoint_uses_event_stream() {
        let response = app()
            .oneshot(
                Request::builder()
                    .uri("/stream")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get("content-type").unwrap(),
            "text/event-stream"
        );
    }
}
