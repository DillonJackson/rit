// helper functions
use std::fs;
use std::io;

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