use super::{tree_item::TreeItem, error::TreeError};
use std::{path::PathBuf, process::Command};
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
        for s in string.replace("\\", "/").split("/") {
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
}

impl TreeModel {
    /// Construct a model using a single tree item.
    pub fn new(item: TreeItem) -> Self {
        TreeModel {root: item.clone(), path: PathVector::new()}
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

    pub fn current_item(&self) -> Result<&TreeItem> {
        let mut current = &self.root;
        for frg in &self.path.path {
            current = current.get_child(frg)?;
        }
        Ok(current)
    }

    pub fn resolve_virtual_path(&self, path: &String) -> Result<Vec<String>> {
        let mut curpath: Vec<String> = Vec::new();
        for s in &self.path.path {
            curpath.push(s.to_string());
        }
        let pathvec = string_to_vec(path);
        for p in pathvec {
            if p == ".." {
                curpath.pop();
            } else {
                curpath.push(p);
            }
        }
        Ok(curpath)
    }

    fn check_path_exists<'a>(&self, path: impl Iterator<Item = &'a String>) -> bool {
        let mut current = &self.root;
        for frg in path {
            current = match current.get_child(frg) {
                Ok(c) => c,
                Err(_) => return false,
            };
        }
        true
    }

    pub fn set_current(&mut self, path: PathVector) -> Result<()> {
        let mut current = &self.root;
        for name in &path.path {
            current = current.get_child_dir(name)?;
        }
        self.path = path;

        Ok(())
    }

    pub fn get_item(&self, path: &String) -> Result<&TreeItem> {
        let pathvec = self.resolve_virtual_path(path)?;
        self.item_at(pathvec)
    }

    pub fn get_item_mut(&mut self, path: &String) -> Result<&mut TreeItem> {
        let pathvec = self.resolve_virtual_path(path)?;
        self.item_at_mut(pathvec)
    }

    fn item_at(&self, pathvec: Vec<String>) -> Result<&TreeItem> {
        let mut cpath = self.path.clone();
        for p in pathvec {
            if p == ".." {
                cpath = cpath.pops(1);
            } else {
                cpath = cpath.join_str(p.to_string());
            }
        }
        let mut item = &self.root;
        for path in &cpath.path {
            item = self.root.get_child(path)?;
        }
        Ok(item)
    }

    fn item_at_mut(&self, pathvec: Vec<String>) -> Result<&mut TreeItem> {
        let mut cpath = self.path.clone();
        for p in pathvec {
            if p == ".." {
                cpath = cpath.pops(1);
            } else {
                cpath = cpath.join_str(p.to_string());
            }
        }
        let mut item = &mut self.root.clone();
        for path in &cpath.path {
            item = item.get_child_mut(path)?;
        }
        Ok(item)
    }

    /// Return the current path.
    pub fn pwd(&self) -> String {
        self.path.as_str()
    }

    pub fn move_forward(&mut self, name: String) -> Result<()> {
        let path = self.path.join_str(name);
        if self.check_path_exists(path.path.iter()) {
            self.path = path;
            Ok(())
        } else {
            Err(TreeError::new("Path does not exist".to_string()))
        }
    }

    pub fn move_backward(&mut self, level: usize) -> Result<()> {
        self.path = self.path.clone().pops(level);
        Ok(())
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
    
    pub fn read_file(&self, path: &String) -> Result<String> {
        let pathvec = self.resolve_virtual_path(path)?;
        let item = self.item_at(pathvec)?;
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
                    format!("{} is not a file.", path),
                )
            )
        }
    }

    /// Open file using default application.
    pub fn open_file(&self, path: &String) -> Result<()> {
        use open::that;
        let pathvec = self.resolve_virtual_path(path)?;
        let item = self.item_at(pathvec)?;
        let path = resolve_path(item.entity.as_ref().unwrap().to_str().unwrap()).unwrap();

        match that(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(TreeError::new(format!("Error opening file: {}", err)))
        }
    }

    /// Get the item at `name` and return its entity path.
    pub fn entity_abspath(&self, path: &String) -> Result<PathBuf> {
        let pathvec = self.resolve_virtual_path(path)?;
        let item = self.item_at(pathvec)?;
        let rpath = item.entity_path();
        match rpath {
            Some(rpath) => match resolve_path(rpath) {
                Ok(path) => Ok(path),
                Err(_) => Err(TreeError::new(format!("Error resolving {}", rpath)))
            }
            None => Err(TreeError::new(format!("No entity found"))),
        }
    }
    pub fn make_directory(&mut self, path: &String) -> Result<()> {
        let mut pathvec = self.resolve_virtual_path(path)?;
        let file_name = match pathvec.pop() {
            Some(name) => name,
            None => return Err(TreeError::new("Path could not be resolved".to_string()))
        };
        let item = match self.item_at_mut(pathvec) {
            Ok(item) => item,
            Err(_) => self.current_item()?,
        };
        item.make_directory(&file_name)?;
        Ok(())
    }

    pub fn remove_child(&mut self, path: &String) -> Result<()> {
        let mut pathvec = self.resolve_virtual_path(path)?;
        let file_name = match pathvec.pop() {
            Some(name) => name,
            None => return Err(TreeError::new("Cannot remove root".to_string()))
        };
        let mut item = self.item_at_mut(pathvec)?;
        item.remove_child(&file_name)
    }

    /// Return the text for "ls" command.
    pub fn ls_simple(&self, path: Option<String>) -> Result<String> {
        let path = match path {
            Some(path) => path,
            None => ".".to_string(),
        };
        let pathvec = self.resolve_virtual_path(&path)?;
        let item = self.item_at(pathvec)?;
        let children: Vec<String> = item.children_names();
        Ok(children.join(" "))
    }

    /// Return the text for "ls --desc" command.
    pub fn ls_with_desc(&self, path: Option<String>) -> Result<String> {
        let path = match path {
            Some(path) => path,
            None => ".".to_string(),
        };
        let pathvec = self.resolve_virtual_path(&path)?;
        let item = self.item_at(pathvec)?;
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
    pub fn add_alias(&mut self, path: Option<&String>, filepath: PathBuf) -> Result<()> {
        if !filepath.exists() {
            return Err(
                TreeError::new(
                    format!("{} does not exist.", filepath.to_str().unwrap()),
                )
            )
        }

        let (dirvec, filename) = match path {
            Some(path) => {
                let mut pathvec = self.resolve_virtual_path(path)?;
                let filename = pathvec.pop().unwrap();
                (pathvec, filename)
            }
            None => {
                let mut pathvec = Vec::new();
                for p in &self.path.path {
                    pathvec.push(p.to_string());
                }
                let filename = filepath.file_name().unwrap().to_str().unwrap().to_string();
                (pathvec, filename)
            }
        };

        let mut item = self.item_at_mut(dirvec)?;
        item.add_item(&filename, filepath)
    }

    pub fn create_new_file(&mut self, path: &String, candidate: PathBuf) -> Result<()> {
        let mut pathvec = self.resolve_virtual_path(path)?;
        if self.check_path_exists(pathvec.iter()) {
            return Err(TreeError::new(format!("{} already exists.", path)))
        }
        let filename = pathvec.pop().unwrap();
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
                    return Err(TreeError::new(format!("{} already exists.", path.to_str().unwrap())))
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
        let mut item = self.item_at_mut(pathvec)?;
        item.add_item(&filename, vpath)
    }

    /// Call external command from the virtual terminal.
    /// let vec = vec!["ls".to_string(), "-l".to_string()];
    /// self.call_command(&vec)
    pub fn call_command(&self, inputs: &Vec<String>) -> Result<()> {
        let mut output: Vec<String> = Vec::new();
        for arg in inputs {
            let pathvec = self.resolve_virtual_path(arg)?;
            let arg = match self.item_at(pathvec) {
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

fn string_to_vec(path: &String) -> Vec<String> {
    path
        .replace("\\", "/")
        .split("/")
        .map(|s| s.to_string())
        .collect::<Vec<String>>()
}
