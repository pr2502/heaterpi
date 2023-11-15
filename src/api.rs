use axum::Json;
use rppal::gpio::Gpio;
use serde::Deserialize;
use tracing::{info, instrument};

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

#[instrument]
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
