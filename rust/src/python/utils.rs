use std::collections::HashMap;
use std::collections::HashSet;
use std::path::PathBuf;

use pyo3::prelude::*;

pub fn extract_to_option_list(
    py: Python<'_>,
    map: &HashMap<String, PyObject>,
    key: &str,
) -> Option<HashSet<Vec<u8>>> {
    let hash_key = String::from(key);
    let option: Option<HashSet<Vec<u8>>> = match map.get(&hash_key) {
        Some(value) => {
            let list: Vec<String> = value.extract(py).unwrap();
            let mut unique_values = HashSet::new();
            for item in list {
                unique_values.insert(item.as_bytes().to_vec());
            }
            Some(unique_values)
        }
        _ => None,
    };
    option
}

pub fn extract_to_option_pathbuf(
    py: Python<'_>,
    map: &HashMap<String, PyObject>,
    key: &str,
) -> Option<PathBuf> {
    let hash_key = String::from(key);
    let option: Option<PathBuf> = map
        .get(&hash_key)
        .map(|value| value.extract::<PathBuf>(py).unwrap());
    option
}

pub fn extract_to_default_string(
    py: Python<'_>,
    map: &HashMap<String, PyObject>,
    key: &str,
    default: &str,
) -> String {
    let hash_key = String::from(key);
    let value = match map.get(&hash_key) {
        Some(value) => value.extract::<String>(py).unwrap(),
        _ => String::from(default),
    };
    value
}

pub fn extract_to_bool(py: Python<'_>, map: &HashMap<String, PyObject>, key: &str) -> bool {
    let hash_key = String::from(key);
    let value = match map.get(&hash_key) {
        Some(value) => value.extract::<bool>(py).unwrap(),
        _ => false,
    };
    value
}

pub fn extract_to_usize(py: Python<'_>, map: &HashMap<String, PyObject>, key: &str) -> usize {
    let hash_key = String::from(key);
    let value = match map.get(&hash_key) {
        Some(value) => value.extract::<usize>(py).unwrap(),
        _ => usize::MAX,
    };
    value
}
