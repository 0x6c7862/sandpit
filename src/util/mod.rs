//! Shared utility functions
use std::fs;
use std::io;
use std::io::prelude::*;

pub fn create_flag(filename: &str, value: &str) -> Result<fs::File, io::Error> {
    // Open file
    let mut file = match fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filename) {
        Ok(val) => val,
        Err(e) => return Err(e),
    };

    // Write content and return handle
    let content = format!("flag{{{}}}\n", value);
    match file.write_all(content.as_bytes()) {
        Ok(_) => Ok(file),
        Err(e) => Err(e),
    }
}
