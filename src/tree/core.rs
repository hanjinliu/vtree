use super::error::{Result, TreeError};

// An item of a tree model.
pub struct TreeItem {
    name: String,
    children: Vec<Box<TreeItem>>,
}

impl TreeItem {
    // True if the tree item is a file.
    pub fn is_file(&self) -> bool {
        self.children.len() == 0
    }

    // True if the tree item is a directory.
    pub fn is_dir(&self) -> bool {
        !self.is_file()
    }

    // Create a new directory named `name`.
    pub fn mkdir(&mut self, name: String) {
        let child = Box::new(
            TreeItem {
                name: name,
                children: Vec::new(),
            }
        );
        self.children.push(child);
    }

    // Remove a directory or a file named `name`.
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
}

impl Iterator for TreeItem {
    type Item = Box<TreeItem>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.children.len() == 0 {
            return None;
        }
        self.children.pop()
    }
}