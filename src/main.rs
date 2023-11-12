use axum::routing::{get, post};
use axum::{Json, Router};
use rppal::gpio::Gpio;
use serde_json::{json, Value};
use tracing::{info, warn};

fn init_logging() {
    use tracing::Level;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{filter, fmt};

    tracing_subscriber::registry()
        .with(fmt::layer().without_time())
        .with(filter::LevelFilter::from_level(Level::INFO))
        .init();
}

async fn root() {}

async fn health() -> Json<Value> {
    Json(json!({}))
}

async fn api_heater_enable(Json(req): Json<Value>) {
    info!(?req);

    let gpio = Gpio::new().unwrap();
    let mut pin = gpio.get(2).unwrap().into_output();

    match req.get("state").unwrap().as_str().unwrap() {
        "on" => {
            pin.set_high();
        }
        "off" => {
            pin.set_low();
        }
        state => warn!(?state, "unknown state"),
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    init_logging();

    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/api/heater/enable", post(api_heater_enable));

    info!("listening on :3000");
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap()
}
