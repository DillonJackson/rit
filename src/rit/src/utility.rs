// helper functions
use std::fs::{self, OpenOptions};
use std::io::{self, Write, Cursor};
use zstd::stream::{encode_all as zstd_compress, decode_all as zstd_decompress};


//removes .rit folder
pub fn repo_remove(path: &str) -> std::io::Result<()> {
    fs::remove_dir_all(path)?;
    Ok(())
}


//initialize .rit 
pub fn init_file_structure()-> io::Result<()> {
    //directories and file names
    const PARENT_DIRECTORY: &str = ".rit";
    const SUB_DIRECTORIES: [&str; 5] = ["hooks", "info", "logs","objects", "refs"];
    const FILES: [&str; 4] = ["HEAD", "config", "discription", "index"];

    //create parent DIR
    fs::create_dir_all(PARENT_DIRECTORY)?;
    

    //create sub DIR
    for dir in SUB_DIRECTORIES.iter() {
        let dir_path = format!("{}/{}", PARENT_DIRECTORY, dir); // Construct the full path
        fs::create_dir_all(&dir_path)?; // Create the subdirectory
    }

    // create files
    for file in FILES.iter() {
        let file_path = format!("{}/{}", PARENT_DIRECTORY, file); // Construct the file path
        fs::File::create(file_path)?; // Create the file
    }

    Ok(())
}

//creates a file
pub fn create_file(file_path: &str, filename: &str, data: Option<&Vec<u8>>) -> io::Result<()> {
    let full_path = format!("{}/{}", file_path, filename);

    // Create the file (this will create an empty file)
    let _ = fs::File::create(&full_path)?;

    if let Some(data) = data {
        // Open the file for writing
        let mut file = OpenOptions::new()
            .write(true)
            .open(&full_path)?;

        // Write the data to the file
        let mut compressed_data = zstd_compress(Cursor::new(data), 3).expect("Failed to compress with zstd");
        file.write_all(&compressed_data)?;
        println!("Data written to {}", full_path);
    }
    
    Ok(())
}


//create a directory
pub fn create_directory(file_path: &str, filename: &str){
    let result = format!("{}/{}", file_path, filename);
    let _ = fs::create_dir_all(result);
}