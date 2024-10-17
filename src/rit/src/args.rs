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

    blob(HashObjectCommand)
}

#[derive(Debug, Args)]
pub struct HashObjectCommand {
    /// The file to store
    pub file: String
}

