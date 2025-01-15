![R](sci-fi rusted letter R.png)
# Rit 
Rit is a streamlined implementation of Git, developed in Rust. It serves as an ideal tool for those looking to manage project source code locally, explore Git functionalities, or gain hands-on experience with Rust development. Whether you are seeking to learn Rust, contribute to an open-source project, or evaluate a new Git alternative, Rit offers a practical starting point.

At present, Rit supports a selection of Git commands. However, not all command options and flags have been implemented. A list of the currently supported commands is provided below.

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


## Reference Link
[A Visual Guide to Git Internals](https://www.freecodecamp.org/news/git-internals-objects-branches-create-repo/)

[Git under the hood](https://coderefinery.github.io/git-intro/under-the-hood/)

[How Git Works Under the Hood](https://www.freecodecamp.org/news/git-under-the-hood)

[How does git detect renames?](https://chelseatroy.com/2020/05/09/question-how-does-git-detect-renames/)

[Learn How Git Works Internally](https://www.gitkraken.com/gitkon/how-does-git-work-under-the-hood#:~:text=Let%E2%80%99s%20take%20a%20look%20at%20how%20Git%20works%20under%20the)

