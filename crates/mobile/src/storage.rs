#![allow(non_snake_case)]

use std::fs;
use std::path::PathBuf;

/// Get the app data directory for persistent storage.
fn data_dir() -> Option<PathBuf> {
    dirs::data_dir().map(|d| d.join("co.superduperai.openwok"))
}

fn jwt_path() -> Option<PathBuf> {
    data_dir().map(|d| d.join("jwt.txt"))
}

pub fn save_jwt(token: &str) {
    if let Some(path) = jwt_path() {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        let _ = fs::write(&path, token);
    }
}

pub fn load_jwt() -> Option<String> {
    let path = jwt_path()?;
    fs::read_to_string(&path).ok().filter(|s| !s.is_empty())
}

pub fn clear_jwt() {
    if let Some(path) = jwt_path() {
        let _ = fs::remove_file(&path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jwt_round_trip() {
        let token = "test-jwt-token-12345";
        save_jwt(token);
        let loaded = load_jwt();
        assert_eq!(loaded.as_deref(), Some(token));
        clear_jwt();
        assert_eq!(load_jwt(), None);
    }
}
