use std::process::{Command, Output};
use std::time::Instant;

use image::{GenericImageView, RgbImage, SubImage};
use tracing::{debug, error};

pub struct Camera {
    yellow: bool,
    green: bool,
    captured: Instant,
}

impl Camera {
    pub fn new() -> Self {
        Camera {
            yellow: false,
            green: false,
            captured: Instant::now(),
        }
    }

    pub fn yellow(&self) -> bool {
        self.yellow
    }

    pub fn green(&self) -> bool {
        self.green
    }

    pub fn age(&self) -> u64 {
        self.captured.elapsed().as_secs()
    }
}

const WIDTH: u32 = 1640;
const HEIGHT: u32 = 1232;

// Measured from a sample image. Thresholds are 50 below the measured value rounded to nearest 5.
const YELLOW_POS: (u32, u32, u32, u32) = (889, 275, 6, 8);
const GREEN_POS: (u32, u32, u32, u32) = (936, 275, 6, 8);
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

#[cfg(feature = "rpi")]
impl Camera {
    pub fn update(&mut self) {
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
                    return;
                };

                self.captured = Instant::now();

                let (x, y, w, h) = YELLOW_POS;
                let yellow = view_avg(img.view(x, y, w, h));
                self.yellow = test(yellow, YELLOW_THRESHOLD);

                let (x, y, w, h) = GREEN_POS;
                let green = view_avg(img.view(x, y, w, h));
                self.green = test(green, GREEN_THRESHOLD);
            }
            Err(err) => {
                error!(?err, "error capturing image");
            }
        }
    }
}

#[cfg(not(feature = "rpi"))]
impl Camera {
    pub fn update(&mut self) {
        self.captured = Instant::now();
        self.yellow = fastrand::bool();
        self.green = fastrand::bool();
    }
}
