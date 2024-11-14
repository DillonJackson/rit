use clap::{Args, Parser, Subcommand};

// Command line interface
#[derive(Debug, Parser)]
#[clap(author, version, name = "Rit", about = "A simple Git like CLI application.")]
pub struct RitArgs {
    #[clap(subcommand)]
    pub command: Commands
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Initialize a new repository
    Init,

    /// Remove the repository
    Remove,

    /// Store the file in the object database and return the key
    HashObject(HashObjectCommand),

    /// Read the file from the object database
    CatFile(CatFileCommand),

    /// Get the file from the object database
    Blob(BlobCommand),

    /// Add the file to the staging area
    Add(AddCommand),

    /// List the contents of a tree object
    LsTree(LsTreeCommand)
}

#[derive(Debug, Args)]
pub struct HashObjectCommand {
    /// The file to store
    pub file: String
}

#[derive(Debug, Args)]
pub struct CatFileCommand {
    /// The key of the file
    pub key: String
}

#[derive(Debug, Args)]
pub struct BlobCommand {
    /// The key of the file
    pub key: String
}

#[derive(Debug, Args)]
pub struct AddCommand {
    /// The file to store
    pub file: String
}

#[derive(Debug, Args)]
pub struct LsTreeCommand {
    /// The key of the tree object
    pub key: String
}