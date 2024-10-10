use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};

// hash the file, then returns the key of the file
fn hash_file(buffer: &Vec<u8>) -> io::Result<String> {
    // Create a new SHA-256 hasher
    let mut hasher = Sha256::new();
    
    // Update the hasher with the file contents
    hasher.update(&buffer);
    
    // Finalize the hash and convert it to a hexadecimal string
    let result = hasher.finalize();
    Ok(format!("{:x}", result))
}

pub fn store_data(file_path: &str) -> io::Result<()>{
    // Open the file in read-only mode
    let mut file = File::open(file_path)?;

    //read the file to buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    //hash the file to obtain the key
    let key = match hash_file(&buffer) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error calculating hash: {}", e);
            return Err(e);
        },
    };
    
    // Print the stored hash value
    println!("SHA-256 hash: {}", key);
    

    //

    Ok(())
}