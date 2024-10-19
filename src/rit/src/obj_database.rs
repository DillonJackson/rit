use sha2::{Sha256, Digest};
use std::io::{self, BufReader, Cursor, BufRead};
use crate::utility::{create_directory,create_file, open_file, uncompress_data};
use json::{JsonValue,parse};
use std::path::Path;
use std::fs;


// Function to create a new file object
fn create_json_obj(file_type: &str, hash_value: &str, filename: &str) -> JsonValue {
    let mut file = JsonValue::new_object();
    file["type"] = file_type.into();
    file["hashvalue"] = hash_value.into();
    file["filename"] = filename.into();
    file
}

pub fn tree_init(dir: &Path)-> io::Result<String> {
    // Create a vector to hold multiple file objects
    let mut files = Vec::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // Check if the entry is a directory or a file
        if path.is_dir() {
            // Get the directory hash
            let dir_hash = tree_init(&path)?;

            //Get the directory name
            let dir_name = {
                if let Some(name) = path.file_name() {
                    name.to_string_lossy().to_string() // Convert and return the name
                } else {
                    String::new() // Default value if None
                }
            };
            
            //stores the directory entry
            let json_obj = create_json_obj("tree", &dir_hash, &dir_name);

            let json_string = json_obj.dump();

            // Convert the String to a Vec<u8>
            let json_bytes: Vec<u8> = json_string.into_bytes();
            files.extend(json_bytes);
            files.push(b'\n'); // Add a newline after each entr
        } else if path.is_file() {

            let file_hash = store_data(&path.display().to_string())?;
    
            // Get the file name
            let file_name = {
                if let Some(name) = path.file_name() {
                    name.to_string_lossy().to_string() // Convert and return the name
                } else {
                    String::new() // Default value if None
                }
            };

            // Creates a json obj
            let json_obj = create_json_obj("blob", &file_hash, &file_name);

            let json_string = json_obj.dump();

            // Must conver the String to a Vec<u8> 
            let json_bytes: Vec<u8> = json_string.into_bytes();

            // Stores data as a 1D vector
            files.extend(json_bytes);
            files.push(b'\n'); // Add a newline after each entr
        }
    }
    
    //hashing the vector
    let key = match store_buffer(&files) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error storing buffer: {}", e);
            return Err(e);
        },
    };

    // println!(" tree SHA-256 hash: {}", key);
    Ok(key)
}

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


//storing a file
pub fn store_file(file_path: &str) -> io::Result<String> {
    // Open the file in read-only mode
    let buffer = open_file(&file_path)?;
    //hash the file to obtain the key
    let key = match store_buffer(&buffer) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error storing buffer: {}", e);
            return Err(e);
        },
    };
    Ok(key)
}

//storing a Vec<u8>
fn store_buffer(buffer: &Vec<u8>) -> io::Result<String>{

    let key = match hash_file(&buffer) {
        Ok(hash_value) => hash_value,
        Err(e) => {
            eprintln!("Error calculating hash: {}", e);
            return Err(e);
        },
    };
    
    // Print the stored hash value
    // println!("SHA-256 hash: {}", key);

    //directory filename
    let sub_dir_name: String = key.chars().take(2).collect();
    // println!("SHA-256 hash first 2 char: {}", sub_dir_name);

    //file filename
    let filename: String = key.chars().skip(2).collect();
    // println!("SHA-256 hash first 2 char: {}", filename);


    //create directory for the objects
    create_directory(".rit/objects", &sub_dir_name)?;

    //creates the file
    let result: String = format!("{}/{}", ".rit/objects", sub_dir_name);
    let result_str: &str = &result;
    // println!("dir {}", result_str);
    create_file(&result_str, &filename, Some(&buffer))?;
    Ok(key)
}

//return the content of a tree
pub fn get_tree(key: &str) -> io::Result<()>{
    let sub_dir_name: String = key.chars().take(2).collect();
    let filename: String = key.chars().skip(2).collect();
    let file_path: String = format!(".rit/objects/{}/{}", sub_dir_name, filename);
    let buffer = open_file(&file_path)?;

    let data = uncompress_data(&buffer)?;
    
    let cursor = Cursor::new(data);
    let reader = BufReader::new(cursor);

    for line in reader.lines() {
        match line {
            Ok(valid_line) => {
                // Parse the line 
                match parse(&valid_line) {
                    Ok(json_obj) => {
                        // Print out the tree data
                        if let (Some(file_type), Some(hashvalue), Some(filename)) = (
                            json_obj["type"].as_str(),
                            json_obj["hashvalue"].as_str(),
                            json_obj["filename"].as_str(),
                        ) {
                            println!("{} {} {}", file_type, hashvalue, filename);
                        } else {
                            println!("Could not retrieve one or more fields.");
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to parse JSON: {}", e);
                    }
                }
            }
            Err(e) => eprintln!("Error reading line: {}", e),
        }
    }
    Ok(())
}


//return the content of a blob
pub fn get_data(key: &str) -> io::Result<Vec<u8>> {
    let sub_dir_name: String = key.chars().take(2).collect();
    let filename: String = key.chars().skip(2).collect();
    let file_path: String = format!(".rit/objects/{}/{}", sub_dir_name, filename);
    let buffer = open_file(&file_path)?;
    match uncompress_data(&buffer) {
        Ok(data) => {
            if let Ok(string_data) = String::from_utf8(data.clone()) {
                println!("Decompressed data: {}", string_data);
            } else {
                println!("Decompressed data (binary): {:?}", data);
            }
            Ok(data)
        },
        Err(e) => Err(e),
    }
}