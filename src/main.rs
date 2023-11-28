use std::net::SocketAddr;
use std::time::Duration;

use axum::http::{Request, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::{debug, error, info, info_span, Level, Span};
use tracing_subscriber::{filter, fmt, prelude::*};

mod api;
mod camera;
mod gpio;

macro_rules! static_file {
    ( $name: literal, $content_type: literal ) => {{
        use axum::http::{header, HeaderName, StatusCode};
        async fn handler() -> (StatusCode, [(HeaderName, &'static str); 1], &'static str) {
            const BODY: &'static str = include_str!(concat!("../static/", $name));
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, $content_type)],
                BODY,
            ) 
        }
        handler
    }};
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {})
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(filter::LevelFilter::from_level(Level::INFO))
        .init();

    let app = Router::new()
        .route("/", get(static_file!("index.html", "text/html")))
        .route("/script.js", get(static_file!("script.js", "application/javascript")))
        .route("/style.css", get(static_file!("style.css", "text/css")))
        .route("/health", get(health))
        .route("/api/heater/enable", post(api::heater_enable))
        .route("/api/heater/state", get(api::heater_state))
        .route("/api/camera", get(api::camera))
        .with_state(api::CameraState::start(Duration::from_secs(5)))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(|request: &Request<_>| {
                    let method = request.method();
                    let path = request.uri();
                    info_span!("request", ?method, ?path)
                })
                .on_request(())
                .on_response(|response: &Response<_>, latency, span: &Span| {
                    let status = response.status();
                    debug!(parent: span, ?status, ?latency, "finished processing request");
                })
                .on_body_chunk(())
                .on_eos(())
                .on_failure(|error, latency, span: &Span| {
                    error!(parent: span, ?error, ?latency, "error processing request");
                }),
        );

    let addr: SocketAddr = "0.0.0.0:3000".parse().unwrap();
    info!(?addr, "listening");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap()
}
