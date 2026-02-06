use crate::model::CompanyState;
use std::fs;
use std::io;
use std::path::Path;

pub fn load_state(path: &Path) -> io::Result<CompanyState> {
    let data = fs::read_to_string(path)?;
    let state =
        serde_json::from_str(&data).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    Ok(state)
}

pub fn save_state(path: &Path, state: &CompanyState) -> io::Result<()> {
    let data = serde_json::to_string_pretty(state)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    fs::write(path, data)
}
