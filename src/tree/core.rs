use std::{ops::Deref, path::PathBuf};

use super::error::{Result, TreeError};
use serde::Deserialize;

/// An item of a tree model.
#[derive(Deserialize, Debug, Clone)]
pub struct TreeItem {
    pub name: String,  // Name of this item.
    children: Vec<Box<TreeItem>>,  // Children of this item.
    pub desc: Option<String>,  // Any description about this model.
    pub entity: Option<PathBuf>,  // The real path to the item.
}

// Implement functions that emulate file system operations.
impl TreeItem {
    /// Create a new empty item with given name.
    fn new(name: String) -> Self {
        TreeItem {
            name,
            children: Vec::new(),
            desc: None,
            entity: None,
        }
    }

    /// Create a new item with file entity at given path.
    fn new_file(name: String, path: PathBuf) -> Self {
        TreeItem {
            name,
            children: Vec::new(),
            desc: None,
            entity: Some(path),
        }
    }

    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tree: TreeItem = serde_json::from_reader(reader)?;
        Ok(tree)
    }

    /// True if the tree item is a file.
    pub fn is_file(&self) -> bool {
        self.entity.as_ref().unwrap().is_file()
    }

    /// True if the tree item is a directory.
    pub fn is_dir(&self) -> bool {
        !self.is_file()
    }

    /// Iterate the children names of this item.
    pub fn iter_children_names(&self) -> impl Iterator<Item = &String> {
        self.children.iter().map(|x| &x.name)
    }

    /// Get the vector of children names.
    pub fn children_names(&self) -> Vec<String> {
        let names:Vec<String> = self.iter_children_names().map(|x| x.clone()).collect();
        names
    }

    pub fn has(&self, name: &String) -> bool {
        for each in &self.children {
            if each.name == *name {
                return true
            }
        }
        false
    }

    /// Get a child item by its name.
    pub fn get_child(&self, name: &String) -> Result<&TreeItem> {
        for each in &self.children {
            if each.name == *name {
                return Ok(each)
            }
        }
        return Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    /// Get a offspring item by its relative path from self.
    /// Unlike `get_child`, input such as "a/b/c" is allowed.
    pub fn get_offspring(&self, name: &String) -> Result<&TreeItem> {
        let mut child = self;
        for filename in name.split('/').into_iter() {
            let mut found = false;
            for each in &child.children {
                if each.name == *filename {
                    child = each.as_ref();
                    found = true;
                    break;
                }
            }
            if !found {
                return Err(TreeError::new(format!("No such file or directory: {}", name)))
            }
        }
        Ok(child)
    }

    /// The mutable version of `get_child`.
    pub fn get_child_mut(&mut self, name: &String) -> Result<&mut TreeItem> {
        for child in &mut self.children {
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

    pub fn new_item(&mut self, name: &String, path: PathBuf) -> Result<()> {
        for child in &self.children {
            if child.name == *name {
                return Err(TreeError::new(format!("File or directory {} already exists.", name)))
            }
        }
        let item = TreeItem::new_file(name.clone(), path);
        let file = Box::new(item);
        self.children.push(file);
        Ok(())
    }

    /// Create a new directory named `name`.
    pub fn mkdir(&mut self, name: &String) -> Result<()>{
        for child in &self.children {
            if child.name == *name {
                return Err(TreeError::new(format!("Directory {} already exists.", name)))
            }
        }
        let child = Box::new(TreeItem::new(name.clone()));
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
/// TreeModel is used to implement moving forward/backward in a tree.
pub struct TreeModel {
    pub root: TreeItem,  // The root tree item.
    pub path: PathVector,  // The current position represented by a vector of keys.
    pub current: TreeItem,  // this field is just for caching.
}

impl TreeModel {
    /// Construct a model using a single tree item.
    pub fn new(item: TreeItem) -> Self {
        TreeModel {root: item.clone(), path: PathVector::new(), current: item}
    }

    /// Construct a model from a json file.
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let item = TreeItem::from_file(path)?;
        Ok(TreeModel::new(item))
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

    pub fn item_at(&self, path: &Vec<String>) -> Result<&TreeItem> {
        let mut current = &self.root;
        for name in path.iter() {
            current = current.get_child(name)?;
        }
        Ok(current)
    }

    pub fn item_at_mut(&mut self, path: &Vec<String>) -> Result<&mut TreeItem> {
        let mut current = &mut self.root;
        for name in path.iter() {
            current = current.get_child_mut(name)?;
        }
        Ok(current)
    }

    pub fn set_item_at(&mut self, path: Vec<String>, item: TreeItem) -> Result<()> {
        // Borrowing happens in the next line so `self.path.path` needs to be evaluated
        // before that.
        let is_current = path == self.path.path;  
        let at = self.item_at_mut(&path)?;
        *at = item;
        if is_current {
            self.current = at.clone();
        }
        Ok(())
    }

    pub fn to_home(&mut self) {
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