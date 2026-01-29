use include_dir::{include_dir, Dir};
use log::{info, warn};
use serde::Deserialize;

static _COURIERS: Dir<'_> = include_dir!("tracking_number_data/couriers/");

#[derive(Deserialize, Debug)]
pub struct Tracking {
    pub courier: String,
    pub service: String,
    pub tracking_number: String,
    pub serial_number: String,
    pub check_digit: String,
    pub tracking_url: String,
}

pub fn track(trk_num: &str) -> Vec<Tracking> {
    let couriers = load_couriers();

    info!("Searching for tracking number: {}", trk_num);

    return couriers;
}

fn load_couriers() -> Vec<Tracking> {
    for file in _COURIERS.files() {
        let contents = match file.contents_utf8() {
            Some(content) => content,
            None => {
                warn!("Unable to read file {}", file.path().display());
                continue;
            }
        };

        let c: Tracking = serde_json::from_str(&contents).unwrap();
        println!("{:?}", c.courier);
    }

    return vec![];
}