//! The public status page and the health answer (see `specs/foundation/personal-server`).
//! Both are static. They never wait for the clone and show nothing about the memory repository.

use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

const PAGE: &str = include_str!("status.html");

pub async fn status_page() -> Response {
    let mut r = (StatusCode::OK, PAGE).into_response();
    let h = r.headers_mut();
    h.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=utf-8"),
    );
    h.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'; style-src 'unsafe-inline'"),
    );
    h.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    r
}

pub async fn health() -> Response {
    let mut r = (StatusCode::OK, "ok").into_response();
    r.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/plain; charset=utf-8"),
    );
    r
}
