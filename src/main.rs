use rppal::gpio::Gpio;
use std::env;

fn main() {
    let arg = env::args().nth(1).expect("usage: heaterpi <low|high>");

    let gpio = Gpio::new().unwrap();
    let mut pin = gpio.get(2).unwrap().into_output();

    match arg.as_str() {
        "low" => pin.set_low(),
        "high" => pin.set_high(),
        _ => panic!("usage: heaterpi <low|high>"),
    }
}
