use include_dir::{include_dir, Dir};
use log::{debug, info, warn};
use pcre2::bytes::Regex;
use serde::{Deserialize, Deserializer};

static COURIERS: Dir<'_> = include_dir!("tracking_number_data/couriers/");

#[derive(Deserialize, Debug)]
pub struct Tracking {
    pub courier: String,
    pub service: String,
    pub tracking_number: String,
    pub tracking_url: String,
}

#[derive(Deserialize, Debug)]
struct Courier {
    name: String,
    #[serde(rename = "courier_code")]
    code: String,
    tracking_numbers: Vec<TrackingNumber>,
}

fn deserialize_pcre<'de, D>(deserializer: D) -> Result<Regex, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawRegex {
        Single(String),
        Multi(Vec<String>),
    }

    let raw = RawRegex::deserialize(deserializer)?;
    let regex_str = match raw {
        RawRegex::Single(s) => s,
        RawRegex::Multi(v)  => v.join("")
    };

    Regex::new(&regex_str).map_err(|e| {
        serde::de::Error::custom(format!("Invalid PCRE2 regex '{}': {}", regex_str, e))
    })
}

#[derive(Deserialize, Debug)]
struct TrackingNumber {
    name: String,
    //#[serde(default)]
    //id: String, // FIXME is this needed?
    #[serde(deserialize_with = "deserialize_pcre")]
    regex: Regex,
    #[cfg(test)]
    test_numbers: TestNumbers,
    #[serde(default)]
    tracking_url: String
}

#[derive(Deserialize, Debug)]
pub struct TestNumbers {
    pub valid: Vec<String>,
    pub invalid: Vec<String>,
}

impl TrackingNumber {
    fn check_format(&self, tracking_number: &str) -> bool {
        let input_bytes = tracking_number.as_bytes();

        debug!("Testing input number {} against regex {:?}", tracking_number, self.regex);

        self.regex.is_match(input_bytes).unwrap_or(false)
    }

    fn check_checksum(&self, tracking_number: &str) -> bool {
        true
    }

    fn check_additional(&self, tracking_number: &str) -> bool {
        true
    }

    fn is_valid(&self, tracking_number: &str) -> bool {
        self.check_format(tracking_number) &&
        self.check_checksum(tracking_number) &&
        self.check_additional(tracking_number)
    }
}

pub fn track(trk_num: &str) -> Option<Tracking> {
    info!("Searching for tracking number: {}", trk_num);

    for c in load_couriers().iter() {
        debug!("Checking {} ({})", c.name, c.code);
        for tn in c.tracking_numbers.iter() {
            if tn.is_valid(trk_num) {
                return Some(Tracking {
                    courier: c.name.to_string(),
                    service: tn.name.to_string(),
                    tracking_number: trk_num.to_string(),
                    tracking_url: "<trk_url>".to_string(),
                });
            }
        }
    }

    return None
}

fn load_couriers() -> Vec<Courier> {
    return COURIERS
        .files()
        .map(|file| {
            debug!("Loading configuration: {}", file.path().display());

            let path = file.path();
            let content = file.contents_utf8().map(|s| {
                serde_json::from_str::<Courier>(s)
            });

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load() {
        let couriers = load_couriers();
        assert_eq!(couriers.len(), 12);
    }

    #[test]
    fn test_valid_numbers() {
        for c in load_couriers().iter() {
            for tn in c.tracking_numbers.iter() {
                for test_num in tn.test_numbers.valid.iter() {
                    assert_eq!(true, tn.is_valid(test_num),
                        "Regex match failed when it should have succeeded. Courier: {} ({}) Tracking Number: {}", c.name, tn.name, test_num);
                }
            }
        }
    }

    #[test]
    fn test_invalid_numbers() {
        for c in load_couriers().iter() {
            for tn in c.tracking_numbers.iter() {
                for test_num in tn.test_numbers.invalid.iter() {
                    assert_eq!(false, tn.is_valid(test_num),
                    "Regex match succeeded when it should have failed. Courier: {} ({}) Tracking Number: {}", c.name, tn.name, test_num);
                }
            }
        }
    }
}