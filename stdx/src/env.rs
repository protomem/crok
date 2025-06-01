use std::env;

const EMPTY_VALUE: &str = "";

pub fn lookup(key: &str) -> Option<String> {
    let value = env::var(key);
    match value {
        Ok(value) => Some(value),
        Err(_) => None,
    }
}

pub fn get_with_default(key: &str, default: &str) -> String {
    match lookup(key) {
        Some(value) => value,
        None => default.to_string(),
    }
}

pub fn get(key: &str) -> String {
    get_with_default(key, EMPTY_VALUE)
}
