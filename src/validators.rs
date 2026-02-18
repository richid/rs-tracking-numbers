
pub trait Validator {
    fn validate(&self, seq: &str, cd: &str) -> bool;
}

pub struct Mod7;

impl Validator for Mod7 {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        let check_digit: u64 = match cd.parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        // Convert entire sequence to integer and take mod 7
        let sequence_int: u64 = match seq.parse() {
            Ok(n) => n,
            Err(_) => return false,
        };

        let calculated = sequence_int % 7;
        calculated == check_digit
    }
}

pub struct Mod10 {
    pub evens_multiplier: u32,
    pub odds_multiplier: u32,
    pub reverse: bool,
}

impl Validator for Mod10 {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        let check_digit: u32 = match cd.parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        let chars: Vec<_> = if self.reverse {
            seq.chars().rev().collect()
        } else {
            seq.chars().collect()
        };

        let mut sum = 0;
        for (i, ch) in chars.iter().enumerate() {
            // Convert character to numeric value
            // For digits: use digit value
            // For letters: (char_code - 3) % 10
            let n = if let Some(d) = ch.to_digit(10) {
                d
            } else {
                ((*ch as u32) - 3) % 10
            };

            let multiplier = if i % 2 == 0 {
                self.evens_multiplier
            } else {
                self.odds_multiplier
            };

            sum += n * multiplier;
        }

        let calculated = (10 - (sum % 10)) % 10;
        calculated == check_digit
    }
}

pub struct SumProduct {
    pub weightings: Vec<u32>,
    pub modulo1: u32,
    pub modulo2: u32,
}

impl Validator for SumProduct {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        let check_digit: u32 = match cd.parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        let digits: Vec<u32> = seq.chars()
            .filter_map(|c| c.to_digit(10))
            .collect();

        if digits.len() != self.weightings.len() {
            return false;
        }

        let sum: u32 = digits.iter()
            .zip(self.weightings.iter())
            .map(|(d, w)| d * w)
            .sum();

        let calculated = (sum % self.modulo1) % self.modulo2;
        calculated == check_digit
    }
}

pub struct S10;

impl Validator for S10 {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        let check_digit: u32 = match cd.parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        let weights = [8, 6, 4, 2, 3, 5, 9, 7];
        let digits: Vec<u32> = seq.chars()
            .filter_map(|c| c.to_digit(10))
            .collect();

        if digits.len() != weights.len() {
            return false;
        }

        let sum: u32 = digits.iter()
            .zip(weights.iter())
            .map(|(d, w)| d * w)
            .sum();

        let remainder = sum % 11;
        let calculated = match 11 - remainder {
            11 => 5,
            10 => 0,
            n => n,
        };

        calculated == check_digit
    }
}

pub struct Luhn;

impl Validator for Luhn {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        let check_digit: u32 = match cd.parse() {
            Ok(d) => d,
            Err(_) => return false,
        };

        let mut sum = 0;
        let mut double = true;

        for ch in seq.chars().rev() {
            let mut n = match ch.to_digit(10) {
                Some(d) => d,
                None => return false,
            };

            if double {
                n *= 2;
                if n > 9 {
                    n -= 9;
                }
            }

            sum += n;
            double = !double;
        }

        let calculated = (10 - (sum % 10)) % 10;
        calculated == check_digit
    }
}

pub struct Mod37_36;

impl Validator for Mod37_36 {
    fn validate(&self, seq: &str, cd: &str) -> bool {
        // From https://esolutions.dpd.com/dokumente/DPD_Parcel_Label_Specification_2.4.1_EN.pdf
        let modulo = 36;

        // Convert character to value (0-9 for digits, 10-35 for A-Z)
        let char_to_val = |ch: char| -> Option<u32> {
            if ch.is_ascii_digit() {
                ch.to_digit(10)
            } else if ch.is_ascii_uppercase() {
                Some((ch as u32) - ('A' as u32) + 10)
            } else {
                None
            }
        };

        // Convert value back to character (0-9 or A-Z)
        let val_to_char = |val: u32| -> char {
            if val < 10 {
                char::from_digit(val, 10).unwrap()
            } else {
                (('A' as u32) + val - 10) as u8 as char
            }
        };

        // Process sequence
        let mut cd_val = modulo;
        for ch in seq.chars() {
            let val = match char_to_val(ch) {
                Some(v) => v,
                None => return false,
            };

            cd_val = val + cd_val;
            if cd_val > modulo {
                cd_val = cd_val - modulo;
            }
            cd_val = cd_val * 2;
            if cd_val > modulo {
                cd_val = cd_val - (modulo + 1);
            }
        }

        cd_val = (modulo + 1) - cd_val;
        if cd_val == modulo {
            cd_val = 0;
        }

        let computed = val_to_char(cd_val);
        let expected = cd.chars().next().unwrap_or('\0').to_ascii_uppercase();

        computed == expected
    }
}
