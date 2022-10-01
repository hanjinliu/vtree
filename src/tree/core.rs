use super::{tree_item::TreeItem, error::TreeError};
use std::{ops::Deref, path::PathBuf};

use super::error::{Result};

// #[derive(Clone, Debug)]
// pub struct VirtualPath {
//     inner: PathBuf,
// }


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
            current = current.get_child_dir(name)?;
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
        let child = self.current.get_child_dir(&name)?;
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
    
    pub fn print_file(&self, arg: &String) -> std::io::Result<()>{
        let item = self.current.get_offspring(&arg).unwrap();
        if item.is_file() {
            let rpath = item.entity.as_ref().unwrap().to_str().unwrap();
            let path = resolve_path(rpath).unwrap();
            let contents = std::fs::read_to_string(&path);
            match contents {
                Ok(value) => { println!("{}", value); }
                Err(err) => { println!("{}", err); }
            }
            Ok(())
        }
        else {
            return Err(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("{} is not a file.", arg),
                )
            )
        }
    }

    pub fn open_file(&self, name: &String) -> std::io::Result<()> {
        use open::that;
        let item = self.current.get_offspring(&name).unwrap();
        let path = resolve_path(item.entity.as_ref().unwrap().to_str().unwrap()).unwrap();
        println!("Opening: {}", path.to_str().unwrap());
        that(path)
    }

    pub fn mkdir(&mut self, name: &String) -> Result<()> {
        let mut item = self.current.clone();
        item.mkdir(&name).unwrap();
        self.set_item_at(self.path.path.clone(), item)
    }

    pub fn rm(&mut self, name: &String) -> Result<()> {
        let mut item = self.current.clone();
        item.rm(name).unwrap();
        self.set_item_at(self.path.path.clone(), item)
    }

    pub fn ls_simple(&self, name: Option<&String>) -> Result<()> {
        let item = match name {
            Some(name) => {
                self.current.get_offspring(&name)?
            }
            None => {
                &self.current
            }
        };
        
        let children: Vec<String> = item.children_names();
        println!("{}", children.join(" "));
        Ok(())
    }

    pub fn add_alias(&mut self, name: Option<&String>, path: PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(
                TreeError::new(
                    format!("{} does not exist.", path.to_str().unwrap()),
                )
            )
        }
        match name {
            Some(name) => self.current.add_item(name, path),
            None => self.current.add_item(
                &path.file_name().unwrap().to_str().unwrap().to_string(), path
            )
        }
    }

}


/// Resolve input path string (must exist) and return a PathBuf with an absolute path.
fn resolve_path(path: &str) -> std::io::Result<PathBuf> {
    if path.starts_with(".") || path.starts_with("/") {
        let curdir = std::env::current_dir()?;
        let path = path.strip_prefix(".").unwrap().strip_prefix("/").unwrap();
        let joined = std::path::Path::new(&curdir).join(path);
        Ok(joined)
    }
    else {
        let joined = std::path::Path::new(&path).to_path_buf();
        Ok(joined)
    }
}