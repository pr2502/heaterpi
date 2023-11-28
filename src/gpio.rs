pub struct Gpio {
    #[cfg(feature = "rpi")]
    pin2: rppal::gpio::OutputPin,

    #[cfg(not(feature = "rpi"))]
    mock_state: bool,
}

#[cfg(feature = "rpi")]
impl Gpio {
    pub fn new() -> Self {
        let gpio = rppal::gpio::Gpio::new().unwrap();
        let pin2 = gpio.get(2).unwrap().into_output();
        Gpio { pin2 }
    }

    pub fn get_state(&self) -> Option<bool> {
        if self.pin2.is_set_low() {
            Some(false)
        } else if self.pin2.is_set_high() {
            Some(true)
        } else {
            None
        }
    }

    pub fn set_state(&mut self, state: bool) {
        if state {
            self.pin2.set_high();
        } else {
            self.pin2.set_low();
        }
    }
}

#[cfg(not(feature = "rpi"))]
impl Gpio {
    pub fn new() -> Self {
        Gpio {
            mock_state: fastrand::bool(),
        }
    }

    pub fn get_state(&self) -> Option<bool> {
        Some(self.mock_state)
    }

    pub fn set_state(&mut self, state: bool) {
        self.mock_state = state;
    }
}
