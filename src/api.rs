use std::{
    process::{Command, Output},
    sync::Arc,
    time::Duration,
};

use axum::{
    body::{Bytes, Full},
    extract::State,
    http::StatusCode,
    response::Response,
    Json,
};
use rppal::gpio::Gpio;
use serde::Deserialize;
use tokio::sync::Mutex;
use tracing::{debug, error, info, instrument};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct HeaterEnableRequest {
    state: HeaterState,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HeaterState {
    On,
    Off,
}

#[instrument(skip_all)]
pub async fn heater_enable(Json(req): Json<HeaterEnableRequest>) {
    let gpio = Gpio::new().unwrap();
    let mut pin = gpio.get(2).unwrap().into_output();

    match req.state {
        HeaterState::On => {
            pin.set_high();
            info!("enabled");
        }
        HeaterState::Off => {
            pin.set_low();
            info!("disabled");
        }
    }
}

pub struct CameraState {
    full_jpg: Mutex<Bytes>,
}

impl CameraState {
    pub fn start(interval: Duration) -> Arc<CameraState> {
        let state = Arc::new(CameraState {
            full_jpg: Mutex::default(),
        });
        tokio::task::spawn_blocking({
            let state = state.clone();
            move || loop {
                // update the state
                let res = Command::new("libcamera-still")
                    .args(["--width", "1640"])
                    .args(["--height", "1232"])
                    .arg("--immediate")
                    .arg("--nopreview")
                    .arg("--flush")
                    .args(["--output", "-"])
                    .output();

                match res {
                    Ok(Output { stdout, stderr, .. }) => {
                        debug!(
                            stderr = %String::from_utf8_lossy(&stderr),
                            "libcamera-still stderr",
                        );
                        *state.full_jpg.blocking_lock() = Bytes::copy_from_slice(&stdout);
                    }
                    Err(err) => {
                        error!(?err, "error capturing image");
                    }
                }

                std::thread::sleep(interval);
            }
        });
        state
    }
}

#[instrument(skip_all)]
pub async fn camera(State(camera): State<Arc<CameraState>>) -> Response<Full<Bytes>> {
    let body = camera.full_jpg.lock().await.clone();
    Response::builder()
        .status(StatusCode::OK)
        .header("Content-Type", "image/jpeg")
        .body(Full::from(body))
        .unwrap()
}
