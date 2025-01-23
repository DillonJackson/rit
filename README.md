![R](/R.png)
# Rit 
Rit is a simplified version of Git built for learning purposes.
Its primary goal is to help developers understand Git's internals and experiment with implementing similar functionality in Rust. Rit is not a replacement for Git and is recommended for educational use only.

At present, Rit supports a selection of Git commands. However, not all command options and flags have been implemented. A list of the currently supported commands is provided below.

## Motivation
Rit was created as an educational project to explore Git's inner workings and to learn Rust. By building a simplified version of Git, this project aims to make Git internals more approachable for developers while showcasing the power of Rust.

## Features
- Local repository initialization and management
- Staging and committing changes
- Object database manipulation
- Basic status reporting

### Planned or Missing Features
- Remote repository interaction (e.g., push, pull, clone)
- Branching and merging
- Rebase and stash functionality

### Working commands
- `rit init` - Initialize a new git repository
- `rit remove` - Removes the repository
- `rit help` - Show the help message
- `rit hash-object` - Store the object in the object database and return the hash
- `rit cat-file` - Print the contents of the object
- `rit blob` - Print the contents of the blob object
- `rit add <file>` - Add a file to the staging area
- `rit ls-tree` - List the contents of a tree object
- `rit commit` - Commit the staged files
- `rit status` - Show the status of the repository


## How to run
```shell
cargo build --release

./target/release/rit <command>
```

## Prerequisites
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version recommended)

### Usage
```shell
rit init

rit add <file>

rit commit
```

## Reference Link
[A Visual Guide to Git Internals](https://www.freecodecamp.org/news/git-internals-objects-branches-create-repo/)

[Git under the hood](https://coderefinery.github.io/git-intro/under-the-hood/)

[How Git Works Under the Hood](https://www.freecodecamp.org/news/git-under-the-hood)

[How does git detect renames?](https://chelseatroy.com/2020/05/09/question-how-does-git-detect-renames/)

[Learn How Git Works Internally](https://www.gitkraken.com/gitkon/how-does-git-work-under-the-hood#:~:text=Let%E2%80%99s%20take%20a%20look%20at%20how%20Git%20works%20under%20the)

## Contributing
Contributions are welcome! Whether it's fixing a bug, adding a feature, or improving the documentation, we would love your help.  
Please follow these steps to contribute:
1. Fork the repository.
2. Create a branch for your feature or bug fix.
3. Submit a pull request with a clear description of your changes.

## License
This project is licensed under the [MIT License](./LICENSE).