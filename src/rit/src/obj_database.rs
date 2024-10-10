use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};


pub fn hash_file(file_path: &str) -> io::Result<String> {
    // Open the file in read-only mode
    let mut file = File::open(file_path)?;
    
    // Create a new SHA-256 hasher
    let mut hasher = Sha256::new();
    
    // Read the entire file into a vector
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    
    // Update the hasher with the file contents
    hasher.update(&buffer);
    
    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}