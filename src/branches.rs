use std::{fs, io::{self, ErrorKind, Write}};
use crate::constants::{head_file_path, refs_dir_path, heads_dir_path};

pub fn init_branches() -> io::Result<()> {
    // Make HEAD file
    let head_file = head_file_path();
    let mut file = std::fs::File::create(&head_file)?;
    file.write_all(b"ref: refs/heads/master\n")?;
    
    // Make refs/heads directory
    let heads_dir = heads_dir_path();
    std::fs::create_dir_all(&heads_dir)?;

    Ok(())
}

pub fn create_banch(branch_name: &str, commit_hash: &str) -> io::Result<()> {
    let branch_file = heads_dir_path().join(branch_name);
    let mut file = std::fs::File::create(&branch_file)?;
    file.write_all(commit_hash.as_bytes())?;
    Ok(())
}

pub fn update_current_branch(commit_hash: &str) -> io::Result<()> {
    let branch_name = get_current_branch_name().expect("HEAD file is not set to a branch");
    let branch_file = heads_dir_path().join(branch_name);
    let mut file = std::fs::File::create(&branch_file)?;
    file.write_all(commit_hash.as_bytes())?;
    Ok(())
}

pub fn get_current_branch_name() -> Option<String> {
    let head_file = head_file_path();
    let head = fs::read_to_string(&head_file).ok()?;
    let head = head.trim();
    let head_parts: Vec<&str> = head.split_whitespace().collect();
    if head_parts.len() < 2 {
        return None;
    }
    let branch_ref = head_parts[1];
    let branch_parts: Vec<&str> = branch_ref.split('/').collect();
    if branch_parts.len() < 3 {
        return None;
    }
    Some(branch_parts[2].to_string())
}

pub fn get_current_branch_commit_hash() -> io::Result<Option<String>> {
    if let Some(branch_name) = get_current_branch_name() {
        get_commit_hash(&branch_name)
    } else {
        Ok(None)
    }
}

pub fn get_commit_hash(branch_name: &str) -> io::Result<Option<String>> {
    let branch_file = heads_dir_path().join(branch_name);
    match fs::read_to_string(&branch_file) {
        Ok(commit_hash) => Ok(Some(commit_hash.trim().to_string())),
        Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(None), // Return None if the branch has no commits
        Err(e) => Err(e), // Propagate other errors
    }
}