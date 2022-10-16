use std::path::PathBuf;

use super::error::{Result, TreeError};
use serde::{Serialize, Deserialize};

/// An item of a tree model.
#[derive(Serialize, Deserialize, Debug, Clone)]
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

    /// Create a new item from a formatted json file.
    pub fn from_file(path: &std::path::Path) -> std::io::Result<Self> {
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        let tree: TreeItem = serde_json::from_reader(reader)?;
        Ok(tree)
    }

    /// True if the tree item is a file.
    pub fn is_file(&self) -> bool {
        match &self.entity {
            Some(path) => path.is_file(),
            None => false,
        }
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

    /// True if the item has a child with given name.
    pub fn has(&self, name: &String) -> bool {
        for each in &self.children {
            if each.name == *name {
                return true
            }
        }
        false
    }

    /// Check if this tree item has a child file with given name.
    // fn has_file(&self, name: &String) -> bool {
    //     for each in &self.children {
    //         if each.name == *name && each.is_file() {
    //             return true
    //         }
    //     }
    //     false
    // }

    /// Check if this tree item has a child directory with given name.
    fn has_dir(&self, name: &String) -> bool {
        for each in &self.children {
            if each.name == *name && each.is_dir() {
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

    /// Get a child directory by its name.
    pub fn get_child_dir(&self, name: &String) -> Result<&TreeItem> {
        for each in &self.children {
            if each.name == *name && each.is_dir() {
                return Ok(each)
            }
        }
        return Err(TreeError::new(format!("No such directory: {}", name)))
    }

    /// Get a offspring item by its relative path from self.
    /// Unlike `get_child`, input such as "a/b/c" is allowed.
    pub fn get_offspring(&self, name: &String) -> Result<&TreeItem> {
        let mut child = self;
        for eachname in name.replace("\\", "/").split('/').into_iter() {
            let mut found = false;
            for each in &child.children {
                if each.name == *eachname {
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

    /// Convert self as a mutable object.
    pub fn as_mut(&mut self) -> &mut TreeItem {
        self
    }

    /// Add a new file named `name` with entity at `path`.
    pub fn add_item(&mut self, name: &String, path: PathBuf) -> Result<()> {
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
    pub fn make_directory(&mut self, name: &String) -> Result<()>{
        if self.has_dir(&name) {
            return Err(TreeError::new(format!("Directory {} already exists.", name)))
        }
        let child = Box::new(TreeItem::new(name.clone()));
        self.children.push(child);
        Ok(())
    }

    /// Remove a directory or a file named `name`.
    pub fn remove_child(&mut self, name: &String) -> Result<()>{
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

    /// Return all the entities.
    pub fn entities(&self) -> Vec<&Box<TreeItem>> {
        let mut values = Vec::new();
        for each in &self.children {
            match &each.entity {
                Some(_) => values.push(each),
                None => {
                    for sub in each.entities() {
                        values.push(&sub);
                    }
                }
            }
        }
        values
    }

    /// Get the absolute path of the entity.
    pub fn entity_path(&self) -> Option<&str> {
        match self.entity.as_ref() {
            Some(ent) => ent.to_str(),
            None => None
        }
    }

    fn _fmt_with_indent(&self, f: &mut std::fmt::Formatter, level: usize) -> std::fmt::Result{
        let blk = "│  ".repeat(level - 1);
        write!(f, "  {}├─ {}\n", blk, self.name)?;
        let nch = self.children.len();
        if nch == 0 {
            return Ok(())
        }
        let mut iter = self.children.iter();
        for _ in 0..nch-1 {
            let child = iter.next().unwrap();
            child._fmt_with_indent(f, level + 1)?;
        }
        let child = iter.next().unwrap();
        child._fmt_with_indent_last(f, level + 1)?;
        Ok(())
    }

    fn _fmt_with_indent_last(&self, f: &mut std::fmt::Formatter, level: usize) -> std::fmt::Result{
        let blk = "│  ".repeat(level - 1);
        write!(f, "  {}└─ {}\n", blk, self.name)?;
        let nch = self.children.len();
        if nch == 0 {
            return Ok(())
        }
        let mut iter = self.children.iter();
        for _ in 0..nch-1 {
            let child = iter.next().unwrap();
            child._fmt_with_indent(f, level + 1)?;
        }
        let child = iter.next().unwrap();
        child._fmt_with_indent_last(f, level + 1)?;
        Ok(())
    }

}

// Implement functions that format the tree item.
impl std::fmt::Display for TreeItem {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}\n", self.name)?;
        let nch = self.children.len();
        let mut iter = self.children.iter();
        for _ in 0..nch-1 {
            let child = iter.next().unwrap();
            child._fmt_with_indent(f, 1)?;
        }
        let child = iter.next().unwrap();
        child._fmt_with_indent_last(f, 1)?;
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
