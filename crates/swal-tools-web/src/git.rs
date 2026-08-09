//! Web Git tools stub. Real implementation lands in 3.07.

/// Clones a git repository into the specified directory.
#[allow(dead_code)]
pub fn clone_repo(_url: &str, _dir: &str) -> Result<(), String> {
    Err("not implemented".into())
}

/// Commits all current changes with the specified commit message.
#[allow(dead_code)]
pub fn commit_all(_msg: &str) -> Result<(), String> {
    Ok(())
}
