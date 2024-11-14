use sha2::{Digest, Sha256};
use std::io;


// hash the file, then returns the key of the file
pub fn hash_data(buffer: &[u8]) -> io::Result<String> {
    // Create a new SHA-256 hasher
    let mut hasher = Sha256::new();

    // Update the hasher with the file contents
    hasher.update(buffer);

    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}