use super::{tree_item::TreeItem, error::TreeError};
use std::{ops::Deref, path::PathBuf, process::Command};
use std::io::Write;
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

    /// Write the tree to a json file at `path`.
    pub fn to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let serialized = serde_json::to_string_pretty(&self.root).unwrap();
        // write
        let mut file = std::fs::File::create(path)?;

        file.write_all(serialized.as_bytes())
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

    pub fn move_to_home(&mut self) -> Result<()> {
        self.set_current(PathVector::new())
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
    
    pub fn read_file(&self, arg: &String) -> Result<String>{
        let item = self.current.get_offspring(&arg)?;
        if item.is_file() {
            let rpath = item.entity.as_ref().unwrap().to_str().unwrap();
            let path = resolve_path(rpath).unwrap();
            let contents = std::fs::read_to_string(&path);
            let out = match contents {
                Ok(value) => value,
                Err(err) => return Err(
                    TreeError::new(
                        format!("{}", err),
                    )
                )
            };
            Ok(out)
        }
        else {
            return Err(
                TreeError::new(
                    format!("{} is not a file.", arg),
                )
            )
        }
    }

    pub fn open_file(&self, name: &String) -> Result<()> {
        use open::that;
        let item = self.current.get_offspring(&name)?;
        let path = resolve_path(item.entity.as_ref().unwrap().to_str().unwrap()).unwrap();

        match that(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(TreeError::new(format!("Error opening file: {}", err)))
        }
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

    // pub fn set_desc(&mut self, path: Option<Vec<String>>, desc: &String) -> Result<()> {
    //     let mut item = match path {
    //         Some(path) => self.item_at_mut(&path)?.clone(),
    //         None => self.current.clone()
    //     };
    //     item.desc = Some(desc.to_string());
    //     let path = match path {
    //         Some(path) => path,
    //         None => self.path.path.clone()
    //     };
    //     self.set_item_at(path, item)
    // }

    fn offspring_at(&self, name: Option<String>) -> Result<&TreeItem> {
        match name {
            Some(name) => {
                self.current.get_offspring(&name)
            }
            None => {
                Ok(&self.current)
            }
        }
    }

    /// Return the text for "ls" command.
    pub fn ls_simple(&self, name: Option<String>) -> Result<String> {
        let item = self.offspring_at(name)?;
        let children: Vec<String> = item.children_names();
        Ok(children.join(" "))
    }

    /// Return the text for "ls --desc" command.
    pub fn ls_with_desc(&self, name: Option<String>) -> Result<String> {
        let item = self.offspring_at(name)?;
        let mut name_vec: Vec<String> = Vec::new();
        let mut desc_vec: Vec<String> = Vec::new();
        for child in item.clone() {
            name_vec.push(child.name);
            desc_vec.push(child.desc.unwrap_or("".to_string()));
        }
        // find longest name to align descriptions
        let mut max_len = 0;
        for name in &name_vec {
            if name.len() > max_len {
                max_len = name.len();
            }
        }
        let mut pair_vec: Vec<String> = Vec::new();
        for (name, desc) in name_vec.iter().zip(desc_vec.iter()) {
            pair_vec.push(format!("{:>width$} {}", name, desc, width=max_len));
        }
        Ok(pair_vec.join("\n"))
    }

    /// Add a alias file to the entity at `path`.
    pub fn add_alias(&mut self, name: Option<String>, path: PathBuf) -> Result<()> {
        if !path.exists() {
            return Err(
                TreeError::new(
                    format!("{} does not exist.", path.to_str().unwrap()),
                )
            )
        }
        let mut item = self.current.clone();
        match name {
            Some(name) => item.add_item(&name, path)?,
            None => item.add_item(
                &path.file_name().unwrap().to_str().unwrap().to_string(), path
            )?
        };
        self.set_item_at(self.path.path.clone(), item)
    }

    pub fn create_new_file(&mut self, name: &String, candidate: PathBuf) -> Result<()> {
        if self.current.has(&name) {
            return Err(TreeError::new(format!("{} already exists.", name)))
        }
        // find unique file name
        let mut path = candidate.clone();
        let vpath_copy = path.clone();
        let stem = vpath_copy.file_stem().unwrap().to_str().unwrap();
        let ext = match vpath_copy.extension() {
            Some(ext) => ".".to_string() + ext.to_str().unwrap(),
            None => {
                "".to_string()
            }
        };
        let mut count = 0;
        // search for unique file name
        let vpath = loop {
            if !path.exists() {
                break path;
            }
            let filename = format!("{}-{}{}", stem, count, ext);
            path = match path.parent() {
                Some(parent) => parent.join(filename),
                None => {
                    return Err(TreeError::new(format!("{} already exists.", name)))
                }
            };
            count += 1;
        };

        // create a hidden file
        match std::fs::File::create(&vpath) {
            Ok(_) => {}
            Err(err) => {
                return Err(TreeError::new(format!("{}: {}", vpath.to_str().unwrap(), err)))
            }
        };
        let mut item = self.current.clone();
        item.add_item(&name, vpath)?;
        self.set_item_at(self.path.path.clone(), item)
    }

    /// Call external command from the virtual terminal.
    /// let vec = vec!["ls".to_string(), "-l".to_string()];
    /// self.call_command(&vec)
    pub fn call_command(&self, inputs: &Vec<String>) -> Result<()> {
        let mut output: Vec<String> = Vec::new();
        for arg in inputs {
            let arg = match self.current.get_offspring(arg) {
                Ok(item) => {
                    match item.entity.as_ref() {
                        Some(entity) => {
                            let entity_path = entity.to_str().unwrap();
                            match resolve_path(entity_path) {
                                Ok(path) => path.to_str().unwrap().to_string(),
                                Err(_) => arg.clone()
                            }
                        }
                        None => arg.clone(),
                    }
                }
                Err(_) => {
                    arg.clone()
                }
            };

            let abspath = resolve_path(&arg);
            match abspath {
                Ok(abspath) => {
                    output.push(abspath.to_str().unwrap().to_string());
                }
                Err(_) => {
                    output.push(arg.to_string());
                }
            }
        }

        // Run command.
        // NOTE: `spawn` is not appropriate for such as `vim`.
        let cmd = if cfg!(target_os = "windows") {
            let mut args = vec!("/C".to_string());
            args.extend(output);
            Command::new("cmd").args(&args).status()
        }
        else {
            let mut args = vec!("-C".to_string());
            args.extend(output);
            Command::new("sh").args(&args).status()
        };
        
        match cmd {
            Ok(_) => Ok(()),
            Err(err) => Err(TreeError::new(format!("Error calling command: {}", err)))
        }
    }
}

/// Resolve input path string (must exist) and return a PathBuf with an absolute path.
/// Input string can be a relative path in the virtual directory or an existing absolute path.
fn resolve_path(path: &str) -> std::io::Result<PathBuf> {
    if path.starts_with(".") || path.starts_with("/") {
        let curdir = std::env::current_dir()?;
        let path = path.strip_prefix(".").unwrap_or(path).strip_prefix("/").unwrap_or(path);
        let joined = std::path::Path::new(&curdir).join(path);
        Ok(joined)
    }
    else {
        let joined = std::path::Path::new(&path).to_path_buf();
        Ok(joined)
    }
}