# Rit (Git made from Rust!)

## Working commands
- `rit init` - Initialize a new git repository
- `rit remove` - Removed the repository
- `rit help` - Show the help message
- `rit add <file>` - Add a file to the staging area
- `rit status` - Show the status of the repository
- `rit commit` - Commit the staged files
- `rit log` - Show the commit history

## How to run
```shell
cargo build --release

./target/release/rit <command>
```

## Usage
```shell
rit init

rit add <file>

rit commit
```