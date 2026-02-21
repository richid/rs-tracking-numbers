mod validators;

use include_dir::{include_dir, Dir};
use lazy_static::lazy_static;
use log::{debug, info, warn};
use pcre2::bytes::Regex;
use serde::{Deserialize, Deserializer};
use serde_json;
use validators::Validator;

static COURIERS: Dir<'_> = include_dir!("tracking_number_data/couriers/");

lazy_static! {
    static ref COURIERS_CACHE: Vec<Courier> = load_couriers();
}

#[derive(Deserialize, Debug)]
pub struct TrackingResult {
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

    // Anchor the pattern to ensure full string match, preventing
    // substring matches that cause misidentification across couriers
    let anchored = if regex_str.starts_with('^') {
        regex_str
    } else {
        format!("^{}$", regex_str)
    };

    Regex::new(&anchored).map_err(|e| {
        serde::de::Error::custom(format!("Invalid PCRE2 regex '{}': {}", anchored, e))
    })
}

#[derive(Deserialize, Debug)]
struct TrackingNumber {
    name: String,
    #[serde(deserialize_with = "deserialize_pcre")]
    regex: Regex,
    #[cfg(test)]
    test_numbers: TestNumbers,
    tracking_url: Option<String>,
    validation: Validation,
    #[serde(default)]
    additional: Vec<AdditionalLookup>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct TestNumbers {
    pub valid: Vec<String>,
    pub invalid: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct Validation {
    checksum: Option<Checksum>,
    serial_number_format: Option<SerialNumberFormat>,
    additional: Option<AdditionalValidation>,
}

#[derive(Debug, Deserialize)]
struct AdditionalValidation {
    exists: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct AdditionalLookup {
    name: String,
    regex_group_name: String,
    lookup: Vec<LookupEntry>,
}

#[derive(Debug, Deserialize)]
struct LookupEntry {
    matches: Option<String>,
    matches_regex: Option<String>,
    #[serde(flatten)]
    _extra: serde_json::Value,  // Catch all other fields
}

#[derive(Debug, Deserialize)]
struct SerialNumberFormat {
    #[serde(default)]
    prepend_if: Option<PrependIf>,
}

#[derive(Debug, Deserialize)]
struct PrependIf {
    matches_regex: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "name")]
enum Checksum {
    #[serde(rename = "mod10")]
    Mod10 {
        evens_multiplier: u32,
        odds_multiplier: u32,
        #[serde(default)]
        reverse: bool,
    },

    #[serde(rename = "mod7")]
    Mod7,

    #[serde(rename = "sum_product_with_weightings_and_modulo")]
    SumProduct {
        weightings: Vec<u32>,
        modulo1: u32,
        modulo2: u32,
    },

    #[serde(rename = "s10")]
    S10,

    #[serde(rename = "mod_37_36")]
    Mod37_36,

    #[serde(rename = "luhn")]
    Luhn,
}

impl TrackingNumber {
    fn extract_captures(&self, tracking_number: &str) -> Option<(String, String)> {
        let captures = self.regex.captures(tracking_number.as_bytes()).ok()??;

        let serial = self.get_named_capture(&captures, "SerialNumber")?;
        let check_digit = self.get_named_capture(&captures, "CheckDigit")?;

        Some((serial, check_digit))
    }

    fn get_named_capture(&self, captures: &pcre2::bytes::Captures, name: &str) -> Option<String> {
        captures.name(name)
            .and_then(|m| std::str::from_utf8(m.as_bytes()).ok())
            .map(|s| s.chars().filter(|c| !c.is_whitespace()).collect())
    }

    fn apply_serial_number_format(&self, serial: &str) -> String {
        if let Some(format) = &self.validation.serial_number_format {
            if let Some(prepend) = &format.prepend_if {
                // Compile regex for prepend_if check
                if let Ok(regex) = Regex::new(&prepend.matches_regex) {
                    if regex.is_match(serial.as_bytes()).unwrap_or(false) {
                        return format!("{}{}", prepend.content, serial);
                    }
                }
            }
        }
        serial.to_string()
    }

    fn check_format(&self, tracking_number: &str) -> bool {
        let input_bytes = tracking_number.as_bytes();
        let result = self.regex.is_match(input_bytes).unwrap_or(false);

        return result;
    }

    fn check_validation(&self, tracking_number: &str) -> bool {
        let Some(checksum) = &self.validation.checksum else {
            return true;
        };

        let Some((serial, check_digit)) = self.extract_captures(tracking_number) else {
            debug!("Failed to extract captures for {}", tracking_number);
            return false;
        };

        let serial = self.apply_serial_number_format(&serial);

        match checksum {
            Checksum::Mod10 { evens_multiplier, odds_multiplier, reverse } => {
                validators::Mod10 {
                    evens_multiplier: *evens_multiplier,
                    odds_multiplier: *odds_multiplier,
                    reverse: *reverse,
                }.validate(&serial, &check_digit)
            }

            Checksum::Mod7 => {
                validators::Mod7.validate(&serial, &check_digit)
            }

            Checksum::SumProduct { weightings, modulo1, modulo2 } => {
                validators::SumProduct {
                    weightings: weightings.clone(),
                    modulo1: *modulo1,
                    modulo2: *modulo2,
                }.validate(&serial, &check_digit)
            }

            Checksum::S10 => {
                validators::S10.validate(&serial, &check_digit)
            }

            Checksum::Luhn => {
                validators::Luhn.validate(&serial, &check_digit)
            }

            Checksum::Mod37_36 => {
                validators::Mod37_36.validate(&serial, &check_digit)
            }
        }
    }

    fn check_additional(&self, tracking_number: &str) -> bool {
        let Some(additional_validation) = &self.validation.additional else {
            return true;
        };

        let Some(exists_list) = &additional_validation.exists else {
            return true;
        };

        let Ok(Some(captures)) = self.regex.captures(tracking_number.as_bytes()) else {
            debug!("Failed to extract captures for additional validation");
            return false;
        };

        for exists_item in exists_list {
            let Some(lookup) = self.additional.iter().find(|l| &l.name == exists_item) else {
                debug!("No lookup found for exists item: {}", exists_item);
                return false;
            };

            let Some(group_value) = self.get_named_capture(&captures, &lookup.regex_group_name) else {
                debug!("Failed to extract regex group: {}", lookup.regex_group_name);
                return false;
            };

            let exists = lookup.lookup.iter().any(|entry| {
                if let Some(ref matches) = entry.matches {
                    if matches == &group_value {
                        return true;
                    }
                }

                if let Some(ref matches_regex) = entry.matches_regex {
                    if let Ok(regex) = Regex::new(matches_regex) {
                        if regex.is_match(group_value.as_bytes()).unwrap_or(false) {
                            return true;
                        }
                    }
                }

                false
            });

            if !exists {
                debug!("Value '{}' not found in lookup table for '{}'", group_value, exists_item);
                return false;
            }
        }

        true
    }

    fn is_valid(&self, tracking_number: &str) -> bool {
        self.check_format(tracking_number) &&
        self.check_validation(tracking_number) &&
        self.check_additional(tracking_number)
    }
}

pub fn track(trk_num: &str) -> Option<TrackingResult> {
    info!("Searching for tracking number: {}", trk_num);

    for courier in COURIERS_CACHE.iter() {
        debug!("Checking {} ({})", courier.name, courier.code);
        for tn in &courier.tracking_numbers {
            if tn.is_valid(trk_num) {
                let tracking_url = tn.tracking_url
                    .as_ref()
                    .map(|url| url.replace("%s", trk_num))
                    .unwrap_or_else(|| String::new());

                return Some(TrackingResult {
                    courier: courier.name.to_string(),
                    service: tn.name.to_string(),
                    tracking_number: trk_num.to_string(),
                    tracking_url,
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
        let result = load_couriers();
        assert_eq!(result.len(), 12);
    }

    #[test]
    fn test_valid_numbers() {
        for courier in load_couriers() {
            for tn in courier.tracking_numbers {
                for valid_num in &tn.test_numbers.valid {
                    assert_eq!(true, tn.is_valid(&valid_num));
                }
            }
        }
    }

    #[test]
    fn test_invalid_numbers() {
        for courier in load_couriers() {
            for tn in courier.tracking_numbers {
                for invalid_num in &tn.test_numbers.invalid {
                    assert_eq!(false, tn.is_valid(&invalid_num));
                }
            }
        }
    }

    #[test]
    fn test_tracking_url() {
        // Test with a UPS tracking number
        let result = track("1Z5R89390357567127");
        assert!(result.is_some(), "Should find UPS tracking number");

        if let Some(tracking) = result {
            assert!(tracking.tracking_url.contains("1Z5R89390357567127"),
                    "URL should contain the tracking number");
            assert!(!tracking.tracking_url.contains("%s"),
                    "URL should not contain the placeholder");
            println!("UPS URL: {}", tracking.tracking_url);
        }

        // Test with a Canada Post tracking number
        let result = track("0073938000549297");
        assert!(result.is_some(), "Should find Canada Post tracking number");

        if let Some(tracking) = result {
            assert!(tracking.tracking_url.contains("0073938000549297"),
                    "URL should contain the tracking number");
            println!("Canada Post URL: {}", tracking.tracking_url);
        }
    }

    #[test]
    fn test_courier_identification() {
        // Verify that regex anchoring prevents substring matches from
        // causing misidentification. Without anchoring, shorter patterns
        // (e.g. FedEx Ground's 15-digit pattern) can match substrings
        // within longer tracking numbers from other couriers.
        let cases = vec![
            ("9400111206206406260787", "United States Postal Service", "USPS 22"),
            ("GM2951173225174494", "DHL", "DHL E-Commerce"),
            ("986578788855", "FedEx", "FedEx Express (12)"),
            ("1Z5R89390357567127", "UPS", "UPS"),
            ("0073938000549297", "Canada Post", "Canada Post (16)"),
        ];

        for (number, expected_courier, expected_service) in cases {
            let result = track(number);
            assert!(result.is_some(), "Should find tracking number: {}", number);
            let tracking = result.unwrap();
            assert_eq!(
                tracking.courier, expected_courier,
                "Wrong courier for {}: expected '{}', got '{}'",
                number, expected_courier, tracking.courier
            );
            assert_eq!(
                tracking.service, expected_service,
                "Wrong service for {}: expected '{}', got '{}'",
                number, expected_service, tracking.service
            );
        }
    }
}
