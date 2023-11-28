use std::sync::Arc;
use std::time::Duration;

use axum::extract::State;
use axum::Json;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{info, instrument};

use crate::camera::Camera;
use crate::gpio::Gpio;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaterEnableRequest {
    state: HeaterState,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "camelCase")]
pub enum HeaterState {
    On,
    Off,
    #[serde(skip_deserializing)]
    Unknown,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaterEnableResponse {
    prev_state: HeaterState,
    changed: bool,
}

#[instrument(skip_all)]
pub async fn heater_enable(Json(req): Json<HeaterEnableRequest>) -> Json<HeaterEnableResponse> {
    let mut gpio = Gpio::new();

    let prev_state = match gpio.get_state() {
        Some(true) => HeaterState::On,
        Some(false) => HeaterState::Off,
        None => HeaterState::Unknown,
    };

    let changed = match (prev_state, req.state) {
        (HeaterState::Off | HeaterState::Unknown, HeaterState::On) => {
            gpio.set_state(true);
            info!("enabled");
            true
        }
        (HeaterState::On | HeaterState::Unknown, HeaterState::Off) => {
            gpio.set_state(false);
            info!("disabled");
            true
        }
        _ => false,
    };

    Json(HeaterEnableResponse {
        prev_state,
        changed,
    })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HeaterStateResponse {
    target: bool,
}

pub async fn heater_state() -> Json<HeaterStateResponse> {
    let gpio = Gpio::new();

    Json(HeaterStateResponse {
        target: gpio.get_state().unwrap_or(false),
    })
}

pub struct CameraState {
    camera: Mutex<Camera>,
}

impl CameraState {
    pub fn start(interval: Duration) -> Arc<CameraState> {
        let state = Arc::new(CameraState {
            camera: Mutex::new(Camera::new()),
        });
        tokio::task::spawn_blocking({
            let state = state.clone();
            move || loop {
                state.camera.blocking_lock().update();
                std::thread::sleep(interval);
            }
        });
        state
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CameraResponse {
    yellow: bool,
    green: bool,
    age: u64,
}

#[instrument(skip_all)]
pub async fn camera(State(state): State<Arc<CameraState>>) -> Json<CameraResponse> {
    let camera = state.camera.lock().await;
    Json(CameraResponse {
        yellow: camera.yellow(),
        green: camera.green(),
        age: camera.age(),
    })
}
