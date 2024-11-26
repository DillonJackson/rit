use core::time;
use std::io;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;
use crate::staging;
use crate::branches;
use crate::tree;
use crate::database;

#[derive(Debug)]
struct Commit {
    tree: String,
    parent: Option<String>,
    committer: String,
    message: String,
    timestamp: u64,
}

impl Commit {
    fn new(tree: String, parent: Option<String>, committer: String, message: String) -> Self {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Commit {
            tree,
            parent,
            committer,
            message,
            timestamp,
        }
    }

    fn serialize(&self) -> Vec<u8> {
        let parent_str = if let Some(ref parent) = self.parent {
            format!("parent {}\n", parent)
        } else {
            String::new()
        };

        format!(
            "tree {}\n{}committer {} {}\n\n{}",
            self.tree,
            parent_str,
            self.committer,
            self.timestamp,
            self.message
        ).into_bytes()
    }

    fn deserialize(data: &[u8]) -> io::Result<Self> {
        let data_str = String::from_utf8_lossy(data);
        let mut tree = String::new();
        let mut parent = None;
        let mut committer = String::new();
        let mut timestamp = 0;
        let mut message = String::new();
        let mut in_message = false;

        for line in data_str.lines() {
            if in_message {
                message.push_str(line);
                message.push('\n');
                continue;
            }

            if line.is_empty() {
                in_message = true;
                continue;
            }

            let mut parts = line.splitn(2, ' ');
            let key = parts.next().unwrap();
            let value = parts.next().unwrap_or("");

            match key {
                "tree" => tree = value.to_string(),
                "parent" => parent = Some(value.to_string()),
                "committer" => {
                    let parts: Vec<&str> = value.split_whitespace().collect();
                    if parts.len() > 2 {
                        committer = parts[..parts.len() - 1].join(" ");
                        timestamp = parts[parts.len() - 1].parse().unwrap_or_default();
                    } else {
                        committer = value.to_string(); // Fallback to the raw value
                        timestamp = 0; // Default timestamp
                    }
                },
                _ => {}
            }
        }

        Ok(Commit {
            tree,
            parent,
            committer,
            message: message.trim_end().to_string(),
            timestamp,
        })
    }
}

pub fn commit(message: &str, commiter:&str) -> io::Result<String> {
    // Get index
    let entries = staging::get_staged_entries()?;
    
    // Get the latest commit hash if there is one
    let latest_commit_hash: Option<String> = branches::get_current_branch_commit_hash()?;

    // Detect if there are no changes to commit, return a message

    // Create a new tree
    let tree_hash = tree::create_tree(&entries)?;

    // Create the commit object and store it in the database
    let commit_hash = create_commit_object(&tree_hash, message, commiter, latest_commit_hash)?;

    // Update the branch to point to the new commit
    branches::update_current_branch(&commit_hash)?;

    Ok(commit_hash)
}

fn create_commit_object(tree_hash: &str, message: &str, commiter: &str, parent_commit_hash: Option<String>) -> io::Result<String> {
    let commit = Commit::new(
        tree_hash.to_string(),
        parent_commit_hash.map(|s| s.to_string()),
        commiter.to_string(),
        message.to_string()
        );

    let commit_data = commit.serialize();

    let commit_hash = database::store_data(&commit_data)?;
    Ok(commit_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_commit_serialization() {
        let tree = "tree_hash".to_string();
        let parent = Some("parent_hash".to_string());
        let committer = "Committer Name <committer@example.com>".to_string();
        let message = "Initial commit".to_string();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let commit = Commit {
            tree: tree.clone(),
            parent: parent.clone(),
            committer: committer.clone(),
            message: message.clone(),
            timestamp,
        };

        let serialized = commit.serialize();
        let expected_serialized = format!(
            "tree {}\nparent {}\ncommitter {} {}\n\n{}",
            tree,
            parent.unwrap(),
            committer,
            timestamp,
            message
        );

        assert_eq!(serialized, expected_serialized.into_bytes());
    }

    #[test]
    fn test_commit_deserialization() {
        let tree = "tree_hash".to_string();
        let parent = "parent_hash".to_string();
        let committer = "Committer Name <committer@example.com>".to_string();
        let message = "Initial commit".to_string();
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();

        let serialized = format!(
            "tree {}\nparent {}\ncommitter {} {}\n\n{}",
            tree,
            parent,
            committer,
            timestamp,
            message
        );

        let deserialized = Commit::deserialize(&serialized.into_bytes()).unwrap();

        assert_eq!(deserialized.tree, tree);
        assert_eq!(deserialized.parent, Some(parent));
        assert_eq!(deserialized.committer, committer);
        assert_eq!(deserialized.message, message);
        assert_eq!(deserialized.timestamp, timestamp);
    }

    #[test]
    fn test_commit_serialization_deserialization() {
        let tree = "tree_hash".to_string();
        let parent = Some("parent_hash".to_string());
        let committer = "Committer Name <committer@example.com>".to_string();
        let message = "Initial commit".to_string();

        let commit = Commit::new(tree.clone(), parent.clone(), committer.clone(), message.clone());
        let serialized = commit.serialize();
        let deserialized = Commit::deserialize(&serialized).unwrap();

        assert_eq!(deserialized.tree, tree);
        assert_eq!(deserialized.parent, parent);
        assert_eq!(deserialized.timestamp, commit.timestamp);
        assert_eq!(deserialized.committer, committer);
        assert_eq!(deserialized.message, message);
    }
}