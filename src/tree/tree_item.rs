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

    /// Create a new item from a formatted string.
    pub fn from_string(s: &String) -> Self {
        serde_json::from_str(s.as_str()).unwrap()
    }

    /// True if the tree item is a file.
    pub fn is_file(&self) -> bool {
        if self.children.len() > 0 {
            return false;
        }
        match &self.entity {
            Some(path) => path.is_file(),
            None => false,
        }
    }

    /// True if the tree item is a directory.
    pub fn is_dir(&self) -> bool {
        !self.is_file()
    }

    pub fn iter_children(&self) -> impl Iterator<Item = &TreeItem> {
        self.children.iter().map(|item| item.as_ref())
    }

    pub fn iter_children_mut(&mut self) -> impl Iterator<Item = &mut TreeItem> {
        self.children.iter_mut().map(|item| item.as_mut())
    }

    /// Iterate the children names of this item.
    pub fn iter_children_names(&self) -> impl Iterator<Item = &String> {
        self.children.iter().map(|x| &x.name)
    }

    /// Get the vector of children names.
    pub fn children_names(&self) -> Vec<String> {
        assert!(self.is_dir());
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
        let (name, nth) = split_nth_item(name);
        if nth < 0 {
            for each in &self.children {
                if each.name == *name {
                    return Ok(each)
                }
            }
        } else {
            let mut count = 0;
            for each in &self.children {
                if each.name == *name {
                    if count == nth {
                        return Ok(each)
                    }
                    count += 1;
                }
            }
            if count > 0 {
                // Found, but not the n-th one.
                return Err(
                    TreeError::new(
                        format!("There are only {} items with name {}", count, name)
                    )
                )
            }
        }
        Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    pub fn get_child_mut(&mut self, name: &String) -> Result<&mut TreeItem> {
        let (name, nth) = split_nth_item(name);
        if nth < 0 {
            for each in self.iter_children_mut() {
                if each.name == *name {
                    return Ok(each)
                }
            }
        } else {
            let mut count = 0;
            for each in self.iter_children_mut() {
                if each.name == *name {
                    if count == nth {
                        return Ok(each)
                    }
                    count += 1;
                }
            }
            if count > 0 {
                // Found, but not the n-th one.
                return Err(
                    TreeError::new(
                        format!("There are only {} items with name {}", count, name)
                    )
                )
            }
        }
        Err(TreeError::new(format!("No such file or directory: {}", name)))
    }

    /// Get a child directory by its name.
    pub fn get_child_dir(&self, name: &String) -> Result<&TreeItem> {
        for each in &self.children {
            if each.name == *name && each.is_dir() {
                return Ok(each)
            }
        }
        return Err(
            TreeError::new(format!("No directory named {} under {}", name, self.name))
        )
    }

    /// Get a child directory by its name.
    pub fn get_child_dir_mut(&mut self, name: &String) -> Result<&mut TreeItem> {
        let item_name = self.name.clone();
        for each in self.iter_children_mut() {
            if each.name == *name && each.is_dir() {
                return Ok(each)
            }
        }
        return Err(
            TreeError::new(format!("No directory named {} under {}", name, item_name))
        )
    }

    /// Convert self as a mutable object.
    pub fn as_mut(&mut self) -> &mut TreeItem {
        self
    }

    /// Add a new file named `name` with entity at `path`.
    pub fn add_new_child(&mut self, name: &String, path: PathBuf) -> Result<()> {
        let item = TreeItem::new_file(name.clone(), path);
        self.add_item(item)
    }

    pub fn add_item(&mut self, item: TreeItem) -> Result<()> {
        let file = Box::new(item);
        self.children.push(file);
        Ok(())
    }

    /// Create a new directory named `name`.
    pub fn make_directory(&mut self, name: &String) -> Result<()> {
        // check if the directory already exists.
        if self.has_dir(&name) {
            return Err(TreeError::new(format!("Directory {} already exists.", name)))
        }
        // check if the directory name is valid.
        if !is_valid_item_name(name) {
            return Err(
                TreeError::new(
                    format!(
                        "Name {} is not a valid directory name. Must not contain
                        \\, /, #, |, \", *, ?, <, > or :", 
                        name
                    )
                )
            )
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

const _INVALID: &str = "\\/#|\"*?<>:";

fn is_valid_item_name(name: &String) -> bool {
    for c in name.chars() {
        if _INVALID.contains(c) {
            return false
        }
    }
    true
}

/// Split a string into a prefix string and a suffix integer.
/// # Examples
/// split_nth_item(&"foo#0".to_string()) -> ("foo", 0)
fn split_nth_item(name: &String) -> (String, i32) {
    let mut iter = name.rsplitn(2, '#');
    let right = iter.next().unwrap().to_string();
    let left = iter.next().unwrap_or("").to_string();
    let nth_result = right.parse::<i32>();
    match nth_result {
        Ok(nth) => (left, nth),
        Err(_) => (name.clone(), -1)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_is_valid() {
        assert!(is_valid_item_name(&"foo".to_string()));
        assert!(is_valid_item_name(&"123.txt".to_string()));
        assert!(!is_valid_item_name(&"2/3".to_string()));
        assert!(!is_valid_item_name(&"3#.json".to_string()));
    }

    #[test]
    fn test_split_nth() {
        assert_eq!(split_nth_item(&"foo.txt#0".to_string()), ("foo.txt".to_string(), 0));
        assert_eq!(split_nth_item(&"foo.txt#4412".to_string()), ("foo.txt".to_string(), 4412));
        assert_eq!(split_nth_item(&"2#4#2".to_string()), ("2#4".to_string(), 2));
    }
}
