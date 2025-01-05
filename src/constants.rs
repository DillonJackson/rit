use std::path::{Path, PathBuf};

pub const DIRECTORY_PATH: &str = ".rit";

pub const INDEX_FILE: &str = "index";

pub const OBJECTS_DIR: &str = "objects";

pub const SOURCE_PATH: &str = "src";

pub const HEAD_FILE: &str = "HEAD";
pub const REFS_DIR: &str = "refs";
pub const HEADS_DIR: &str = "heads";

pub fn directory_path() -> PathBuf {
    Path::new(DIRECTORY_PATH).to_path_buf()
}

pub fn source_path() -> PathBuf {
    Path::new(SOURCE_PATH).to_path_buf()
}

pub fn index_file_path() -> PathBuf {
    directory_path().join(INDEX_FILE)
}

pub fn objects_dir_path() -> PathBuf {
    directory_path().join(OBJECTS_DIR)
}

pub fn head_file_path() -> PathBuf {
    directory_path().join(HEAD_FILE)
}

pub fn refs_dir_path() -> PathBuf {
    directory_path().join(REFS_DIR)
}

pub fn heads_dir_path() -> PathBuf {
    refs_dir_path().join(HEADS_DIR)
}


// Object database types
pub const BLOB: &str = "blob";
pub const TREE: &str = "tree";
pub const COMMIT: &str = "commit";
