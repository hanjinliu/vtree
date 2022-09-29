use std::ops::Deref;

use super::error::{Result, TreeError};
use serde::Deserialize;

/// An item of a tree model.
#[derive(Deserialize, Debug, Clone)]
pub struct TreeItem {
    pub name: String,
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

    pub fn get_child(&self, name: &String) -> Result<&TreeItem> {
        for child in &self.children {
            if child.name == *name {
                return Ok(child)
            }
        }
        return Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    pub fn update_child(&mut self, name: &String, new_child: TreeItem) -> Result<()> {
        for child in &mut self.children {
            if child.name == *name {
                *child = Box::new(new_child);
                return Ok(())
            }
        }
        return Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    /// Create a new directory named `name`.
    pub fn mkdir(&mut self, name: &String) -> Result<()>{
        for child in &self.children {
            if child.name == *name {
                return Err(TreeError::new(format!("Directory {} already exists.", name)))
            }
        }
        let child = Box::new(
            TreeItem {
                name: name.clone(),
                children: Vec::new(),
            }
        );
        self.children.push(child);
        Ok(())
    }

    /// Remove a directory or a file named `name`.
    pub fn rm(&mut self, name: &String) -> Result<()>{
        let mut index = 0;
        for child in &self.children {
            if child.name == *name {
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

#[derive(Clone)]
pub struct PathVector {
    pub path: Vec<String>,
}

impl PathVector {
    pub fn new() -> Self {
        PathVector{path: Vec::new()}
    }

    pub fn from_vec(strings: Vec<String>) -> Self {
        PathVector{path: strings}
    }

    pub fn from_string(string: String) -> Self {
        let mut vec = Vec::new();
        for s in string.split("/") {
            vec.push(s.to_string());
        }
        PathVector{path: vec}
    }

    /// Create a new path vector by extending the existing path with a string. This
    /// function is an immutable operation.
    pub fn join_str(&self, name: String) -> Self {
        let mut vec = self.path.clone();
        vec.push(name);
        Self::from_vec(vec)
    }

    /// Create a new path vector by extending the existing path with another path
    /// vector. This function is an immutable operation.
    pub fn join_path(&self, path: &PathVector) -> Self {
        let mut vec = self.path.clone();
        vec.extend(path.path.clone());
        Self::from_vec(vec)
    }

    pub fn pops(self, level: usize) -> Self {
        let mut vec = self.path.clone();
        let npop = level.min(vec.len());
        for _ in 0..npop {
            vec.pop();
        }
        Self::from_vec(vec)
    }

    pub fn as_str(&self) -> String {
        self.path.join("/")
    }
}

/// A struct with a tree and the current position.
/// TreeSystem is used to implement moving forward/backward in a tree.
pub struct TreeSystem {
    pub root: TreeItem,  // The root tree item.
    pub path: PathVector,  // The current position represented by a vector of keys.
    pub current: TreeItem,  // this field is just for caching.
}

impl TreeSystem {
    pub fn new(item: TreeItem) -> Self {
        TreeSystem {root: item.clone(), path: PathVector::new(), current: item}
    }

    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let item = TreeItem::from_file(path)?;
        Ok(TreeSystem::new(item))
    }

    pub fn set_current(&mut self, path: PathVector) -> Result<()> {
        let mut current = &self.root;
        for name in &path.path {
            current = current.get_child(name)?;
        }
        self.path = path;
        self.current = current.deref().clone();

        Ok(())
    }

    pub fn home(&mut self) {
        self.current = self.root.clone();
        self.path = PathVector::new();
    }

    /// Return the current path.
    pub fn pwd(&self) -> String {
        self.path.as_str()
    }

    pub fn move_forward(&mut self, name: String) -> Result<()> {
        let child = self.current.get_child(&name)?;
        self.current = child.clone();
        self.path = self.path.join_str(name);
        Ok(())
    }

    pub fn move_backward(&mut self, level: usize) -> Result<()> {
        let path = self.path.clone().pops(level);
        self.set_current(path)
    }

    pub fn move_by_string(&mut self, path: &String) -> Result<()> {
        for s in path.split("/") {
            if s == "" {
                continue;
            }
            if s == ".." {
                self.move_backward(1)?;
            } else {
                self.move_forward(s.to_string())?;
            }
        }
        Ok(())
    }

    pub fn as_prefix(&self) -> String {
        let name = &self.root.name;
        let path = self.path.as_str();
        format!("/[{}]/{} > ", name, path)
    }
}