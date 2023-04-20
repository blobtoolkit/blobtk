use indicatif::{ProgressBar, ProgressStyle};

use rust_decimal::prelude::*;

pub mod compact_float {
    //! rounds a float to 3 decimal places, when serialized into a str, such as for JSON
    //! offers space savings when such such precision is not needed.
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(float: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{:.3}", float);
        let parsed = s.parse::<f64>().unwrap();
        serializer.serialize_f64(parsed)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer)
    }
}

pub fn format_si(value: &f64, digits: u32) -> String {
    fn set_suffix(thousands: i8) -> String {
        const POSITIVE: [&str; 9] = ["", "k", "M", "G", "T", "P", "E", "Z", "Y"];
        const NEGATIVE: [&str; 9] = ["", "m", "μ", "p", "n", "f", "a", "z", "y"];
        let suffix = if thousands < 0 && thousands >= -9 {
            NEGATIVE[(thousands * -1) as usize]
        } else if thousands >= 0 && thousands <= 9 {
            POSITIVE[thousands as usize]
        } else {
            ""
        };
        suffix.to_string()
    }

    let magnitude = (value.clone()).log10() as i8;
    let thousands = magnitude / 3;
    let prefix = if thousands < 0 {
        value.clone()
    } else {
        value.clone() / 10u32.pow(3 * thousands as u32) as f64
    };
    let d = Decimal::from_f64_retain(prefix).unwrap();
    let rounded = d
        .round_sf_with_strategy(digits, RoundingStrategy::MidpointAwayFromZero)
        .unwrap()
        .normalize()
        .to_string();

    let suffix = set_suffix(thousands);
    format!("{}{}", rounded, suffix)
}

pub fn indexed_sort<T: Ord>(list: &[T]) -> Vec<usize> {
    let mut indices = (0..list.len()).collect::<Vec<_>>();
    indices.sort_by_key(|&i| &list[i]);
    indices.reverse();
    indices
}

pub fn styled_progress_bar(total: usize, message: &str) -> ProgressBar {
    let progress_bar = ProgressBar::new(total as u64);
    let format_string = format!(
        "[+]\t{}: {{bar:40.cyan/blue}} {{pos:>7}}/{{len:12}}",
        message
    );

    let pb_style_result = ProgressStyle::with_template(format_string.as_str());
    let pb_style = match pb_style_result {
        Ok(style) => style,
        Err(error) => panic!("Problem with the progress bar: {:?}", error),
    };
    progress_bar.set_style(pb_style);
    progress_bar
}

pub fn linear_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]) as f64 / (domain[1] - domain[0]) as f64;
    (range[1] - range[0]) * proportion + range[0]
}

pub fn linear_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]) / (domain[1] - domain[0]);
    (range[1] - range[0]) * proportion + range[0]
}

pub fn log_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion =
        ((value - domain[0]) as f64).log10() / ((domain[1] - domain[0]) as f64).log10();
    (range[1] - range[0]) * proportion + range[0]
}

pub fn log_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]).log10() / (domain[1] - domain[0]).log10();
    (range[1] - range[0]) * proportion + range[0]
}

pub fn sqrt_scale(value: usize, domain: &[usize; 2], range: &[f64; 2]) -> f64 {
    let proportion = ((value - domain[0]) as f64).sqrt() / ((domain[1] - domain[0]) as f64).sqrt();
    (range[1] - range[0]) * proportion + range[0]
}

pub fn sqrt_scale_float(value: f64, domain: &[f64; 2], range: &[f64; 2]) -> f64 {
    let proportion = (value - domain[0]).sqrt() / (domain[1] - domain[0]).sqrt();
    (range[1] - range[0]) * proportion + range[0]
}
