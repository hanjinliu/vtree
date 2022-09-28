use super::error::{Result, TreeError};
use serde::Deserialize;

/// An item of a tree model.
#[derive(Deserialize, Debug)]
pub struct TreeItem {
    name: String,
    children: Vec<Box<TreeItem>>,
}

// Implement functions that emulate file system operations.
impl TreeItem {
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tree: TreeItem = serde_json::from_reader(reader)?;
        Ok(tree)
    }

    /// True if the tree item is a file.
    pub fn is_file(&self) -> bool {
        self.children.len() == 0
    }

    /// True if the tree item is a directory.
    pub fn is_dir(&self) -> bool {
        !self.is_file()
    }

    /// Create a new directory named `name`.
    pub fn mkdir(&mut self, name: String) {
        let child = Box::new(
            TreeItem {
                name: name,
                children: Vec::new(),
            }
        );
        self.children.push(child);
    }

    /// Remove a directory or a file named `name`.
    pub fn rm(&mut self, name: String) -> Result<()>{
        let mut index = 0;
        for child in &self.children {
            if child.name == name {
                self.children.remove(index);
                return Ok(())
            }
            index += 1;
        }
        return Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    fn _fmt_with_indent(&self, f: &mut std::fmt::Formatter, level: usize) -> std::fmt::Result{
        let blk = " ".repeat(level * 4 - 3);
        write!(f, " {}|- {}\n", blk, self.name)?;
        for child in &self.children {
            child._fmt_with_indent(f, level + 1)?;
        }
        Ok(())
    }
}

// Implement functions that format the tree item.
impl std::fmt::Display for TreeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}\n", self.name)?;
        for child in &self.children {
            child._fmt_with_indent(f, 1)?;
        }
        Ok(())
    }
}

// Implement iterator for the tree item.
impl Iterator for TreeItem {
    type Item = Box<TreeItem>;

    /// Iterate over the children of the tree item.
    fn next(&mut self) -> Option<Self::Item> {
        if self.children.len() == 0 {
            return None;
        }
        self.children.pop()
    }
}