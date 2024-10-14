// helper functions
use std::fs::File;
use std::fs::{self, OpenOptions};
use std::io::{self, Write, Cursor, Read};
use zstd::stream::{encode_all as zstd_compress, decode_all as zstd_decompress};

pub fn open_file(file_path: &str) -> io::Result<Vec<u8>> {
    // Open the file in read-only mode
    let mut file = match File::open(file_path) {
        Ok(f) => f, // File created successfully
        Err(e) => {
            eprintln!("Error opening file: {}", e); // Log the error
            return Err(e);
        }
    };

    // Read the file into the buffer
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;
    Ok(buffer)
}

//removes .rit folder
pub fn repo_remove(path: &str) -> io::Result<()> {
    match fs::remove_dir_all(path) {
        Ok(_) => {
            //println!("Successfully removed repo: {}", path);
            Ok(())
        }
        Err(e) => {
            eprintln!("Error removing repo '{}': {}", path, e);
            return Err(e);
        }
    }
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
// pass in "None" to data when not writing to file
pub fn create_file(file_path: &str, filename: &str, data: Option<&Vec<u8>>) -> io::Result<()> {
    let full_path = format!("{}/{}", file_path, filename);

    // Create the file (this will create an empty file)
    fs::File::create(&full_path)?;

    if let Some(data) = data {
        // Open the file for writing
        let mut file = OpenOptions::new()
            .write(true)
            .open(&full_path)?;

        // Write the data to the file
        let compressed_data = zstd_compress(Cursor::new(data), 3).expect("Failed to compress with zstd");
        file.write_all(&compressed_data)?;
        //println!("Data written to {}", full_path);
    }

    Ok(())
}


//create a directory
pub fn create_directory(file_path: &str, dir_name: &str) -> io::Result<()>{
    let result = format!("{}/{}", file_path, dir_name);
    fs::create_dir_all(&result)?;
    Ok(())
}