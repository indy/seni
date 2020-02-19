use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub fn get_uuid() -> String {
    Uuid::new_v4()
        .to_simple()
        .encode_lower(&mut Uuid::encode_buffer())
        .to_string()
}

pub fn get_session_filepath(session_path: &str, app_name: &str, uuid: &str) -> PathBuf {
    let filename = app_name.to_owned() + "-session-" + &uuid;
    let session_file = PathBuf::from(session_path).join(filename);

    session_file
}

pub fn get_session_id(session_filepath: &Path) -> Option<String> {
    if session_filepath.is_file() {
        fs::read_to_string(session_filepath).ok()
    } else {
        None
    }
}

pub fn write_session_id(session_filepath: &Path, id: &str) -> Result<()> {
    Ok(fs::write(session_filepath, id)?)
}
