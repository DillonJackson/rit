use std::fs::File;
use std::io::{self, Read};
use sha2::{Sha256, Digest};
use crate::utility::{create_directory,create_file};

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

    //directory filename
    let sub_dir_name: String = key.chars().take(2).collect();
    // println!("SHA-256 hash first 2 char: {}", sub_dir_name);

    //file filename
    let filename: String = key.chars().skip(2).collect();
    // println!("SHA-256 hash first 2 char: {}", filename);


    //create directory for the objects
    create_directory(".rit/objects", &sub_dir_name);

    //creates the file
    let result: String = format!("{}/{}", ".rit/objects", sub_dir_name);
    let result_str: &str = &result;
    // println!("dir {}", result_str);
    let _ = create_file(&result_str, &filename, Some(&buffer));
    Ok(())
}

//    let new_str: &str = format!("{}/{}", "./rit", sub_dir_name);
//    create_file(&new_str, &filename);