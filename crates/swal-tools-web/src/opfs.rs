//! OPFS tools stub. Real implementation lands in 3.06.

/// Reads a file from the Origin Private File System (OPFS).
#[allow(dead_code)]
pub fn read_file(_path: &str) -> Result<String, String> {
    Err("not implemented".into())
}

/// Writes content to a file in the Origin Private File System (OPFS).
#[allow(dead_code)]
pub fn write_file(_path: &str, _content: &str) -> Result<(), String> {
    Ok(())
}
