use std::process::{Command, Output};
use std::sync::Arc;
use std::time::{Duration, Instant};

use axum::extract::State;
use axum::Json;
use image::{GenericImageView, RgbImage, SubImage};
use rppal::gpio::Gpio;
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize)]
pub struct HeaterEnableResponse {}

#[instrument(skip_all)]
pub async fn heater_enable(Json(req): Json<HeaterEnableRequest>) -> Json<HeaterEnableResponse> {
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
    Json(HeaterEnableResponse {})
}

pub struct CameraState {
    yellow: bool,
    green: bool,
    captured: Instant,
}

const WIDTH: u32 = 1640;
const HEIGHT: u32 = 1232;

// Measured from a sample image. Thresholds are 50 below the measured value rounded to nearest 5.
const YELLOW_POS: (u32, u32, u32, u32) = (881, 269, 9, 10);
const GREEN_POS: (u32, u32, u32, u32) = (927, 268, 9, 10);
const YELLOW_THRESHOLD: (u64, u64, u64) = (155, 105, 0);
const GREEN_THRESHOLD: (u64, u64, u64) = (145, 180, 0);

fn view_avg(view: SubImage<&RgbImage>) -> (u64, u64, u64) {
    let count = view.pixels().count() as u64;
    let sum = view.pixels().fold((0, 0, 0), |acc, (_, _, pix)| {
        (
            acc.0 + u64::from(pix[0]),
            acc.1 + u64::from(pix[1]),
            acc.2 + u64::from(pix[2]),
        )
    });
    (sum.0 / count, sum.1 / count, sum.2 / count)
}

fn test(val: (u64, u64, u64), thresh: (u64, u64, u64)) -> bool {
    val.0 > thresh.0 && val.1 > thresh.1 && val.2 > thresh.2
}

impl CameraState {
    pub fn start(interval: Duration) -> Arc<Mutex<CameraState>> {
        let state = Arc::new(Mutex::new(CameraState {
            yellow: false,
            green: false,
            captured: Instant::now(),
        }));
        tokio::task::spawn_blocking({
            let state = state.clone();
            move || loop {
                std::thread::sleep(interval);

                // update the state
                let res = Command::new("libcamera-still")
                    .args(["--width", &format!("{WIDTH}")])
                    .args(["--height", &format!("{HEIGHT}")])
                    .arg("--immediate")
                    .arg("--nopreview")
                    .arg("--flush")
                    .args(["--encoding", "rgb"])
                    .args(["--output", "-"])
                    .output();

                match res {
                    Ok(Output { stdout, stderr, .. }) => {
                        debug!(
                            stderr = %String::from_utf8_lossy(&stderr),
                            "libcamera-still stderr",
                        );

                        let Some(img) = RgbImage::from_raw(WIDTH, HEIGHT, stdout) else {
                            error!("parse raw image from libcamera output");
                            continue;
                        };

                        let mut state = state.blocking_lock();
                        state.captured = Instant::now();

                        let (x, y, w, h) = YELLOW_POS;
                        let yellow = view_avg(img.view(x, y, w, h));
                        state.yellow = test(yellow, YELLOW_THRESHOLD);

                        let (x, y, w, h) = GREEN_POS;
                        let green = view_avg(img.view(x, y, w, h));
                        state.green = test(green, GREEN_THRESHOLD);
                    }
                    Err(err) => {
                        error!(?err, "error capturing image");
                    }
                }
            }
        });
        state
    }
}

#[derive(Debug, Serialize)]
pub struct CameraResponse {
    yellow: bool,
    green: bool,
    age: u64,
}

#[instrument(skip_all)]
pub async fn camera(State(camera): State<Arc<Mutex<CameraState>>>) -> Json<CameraResponse> {
    let camera = camera.lock().await;
    Json(CameraResponse {
        yellow: camera.yellow,
        green: camera.green,
        age: camera.captured.elapsed().as_secs(),
    })
}
