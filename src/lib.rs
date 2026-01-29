use include_dir::{include_dir, Dir};
use log::{info, warn};
use serde::Deserialize;

static COURIERS: Dir<'_> = include_dir!("tracking_number_data/couriers/");

#[derive(Deserialize, Debug)]
pub struct Tracking {
    pub courier: String,
    pub service: String,
    pub tracking_number: String,
    pub serial_number: String,
    pub check_digit: String,
    pub tracking_url: String,
}

#[derive(Deserialize, Debug)]
struct Courier {
    name: String,
    #[serde(rename = "courier_code")]
    code: String
}

pub fn track(trk_num: &str) -> Vec<Tracking> {
    info!("Searching for tracking number: {}", trk_num);

    let couriers = load_couriers();

    for c in couriers.iter() {
        println!("Checking {}({})", c.name, c.code);
    }

    return vec![];
}

fn load_couriers() -> Vec<Courier> {
    return COURIERS
        .files()
        .map(|file| {
            let path = file.path();
            let content = file.contents_utf8().map(|s| {
                serde_json::from_str::<Courier>(s)
            });

            info!("Attempting to read {}", path.display());

            (path, content)
        })
        .inspect(|(path, content)| match content {
            Some(Ok(_))  => (),
            Some(Err(e)) => warn!("Warning: '{}' is invalid JSON: {}", path.display(), e),
            None         => warn!("Warning: '{}' is not valid UTF-8", path.display()),
        })
        .filter_map(|(_, content)| content?.ok())
        .collect();
}