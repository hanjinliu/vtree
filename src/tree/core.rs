use super::{tree_item::TreeItem, error::TreeError};
use std::{path::PathBuf, process::Command};
use std::io::Write;
use super::error::Result;


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
    pub fn join_str(&mut self, name: String) {
        self.path.push(name);
    }

    /// Create a new path vector by extending the existing path with another path
    /// vector. This function is an immutable operation.
    pub fn join_path(&self, path: &PathVector) -> Self {
        let mut vec = self.path.clone();
        vec.extend(path.path.clone());
        Self::from_vec(vec)
    }

    pub fn pops(&mut self, level: usize) {
        let npop = level.min(self.path.len());
        for _ in 0..npop {
            self.path.pop();
        }
    }

    /// Convert path vector into the formatted path string.
    pub fn to_string(&self) -> String {
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

    pub fn from_string(s: &str) -> Self {
        let item = TreeItem::from_string(&s.to_string());
        TreeModel::new(item)
    }

    /// Write the tree to a json file at `path`.
    pub fn to_file(&self, path: &std::path::Path) -> std::io::Result<()> {
        let serialized = serde_json::to_string_pretty(&self.root).unwrap();
        // write
        let mut file = std::fs::File::create(path)?;

        file.write_all(serialized.as_bytes())
    }

    /// Get the current tree item.
    pub fn current_item(&self) -> Result<&TreeItem> {
        let mut current = &self.root;
        for frg in &self.path.path {
            current = current.get_child_dir(frg)?;
        }
        Ok(current)
    }

    /// Get the current tree item as a mutable reference.
    pub fn current_item_mut(&mut self) -> Result<&mut TreeItem> {
        let mut current = &mut self.root;
        for frg in &self.path.path {
            current = current.get_child_dir_mut(frg)?;
        }
        Ok(current)
    }

    pub fn resolve_virtual_path(&self, path: &String) -> Result<Vec<String>> {
        let mut curpath: Vec<String> = Vec::new();
        if !path.starts_with("~") {
            for s in &self.path.path {
                curpath.push(s.to_string());
            }
        }

        let pathvec = path
            .replace("\\", "/")
            .split("/")
            .map(|s| s.to_string())
            .filter(|s| s != "" && s != "." && s != "~")
            .collect::<Vec<String>>();

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

    /// Get tree item at `path`.
    pub fn get_item(&self, path: &String) -> Result<&TreeItem> {
        let pathvec = self.resolve_virtual_path(path)?;
        self.item_at(pathvec)
    }

    fn solve_path_vec(&self, pathvec: &Vec<String>) -> PathVector {
        let mut cpath = 
            if pathvec.get(0) == Some(&"~".to_string()) {
                PathVector::new()
            } else {
                self.path.clone()
            };

        for p in pathvec {
            if p == ".." {
                cpath.pops(1);
            } else {
                cpath.join_str(p.to_string());
            }
        }
        cpath
    }

    /// Get tree item at `pathvec`.
    fn item_at(&self, pathvec: Vec<String>) -> Result<&TreeItem> {
        let cpath = self.solve_path_vec(&pathvec);
        let mut item = &self.root;
        for path in &cpath.path {
            item = item.get_child(path)?;
        }
        Ok(item)
    }
    
    /// Get tree item at `pathvec` as a mutable reference.
    fn item_at_mut(&mut self, pathvec: Vec<String>) -> Result<&mut TreeItem> {
        let cpath = self.solve_path_vec(&pathvec);
        let mut item = &mut self.root;
        for path in &cpath.path {
            item = item.get_child_mut(path)?;
        }
        Ok(item)
    }

    /// Get directory tree item at `pathvec`.
    fn dir_item_at(&self, pathvec: &Vec<String>) -> Result<&TreeItem> {
        let mut cpath = self.solve_path_vec(&pathvec);
        let mut item = &self.root;
        let last = cpath.path.pop().unwrap();
        for path in &cpath.path {
            item = item.get_child_dir(path)?;
        }
        let item = item.get_child_dir(&last)?;
        assert!(item.is_dir());
        Ok(item)
    }

    
    /// Get directory tree item at `pathvec` as a mutable reference.
    fn dir_item_at_mut(&mut self, pathvec: &Vec<String>) -> Result<&mut TreeItem> {
        let mut cpath = self.solve_path_vec(&pathvec);
        let mut item = &mut self.root;
        let last = cpath.path.pop().unwrap();
        for path in &cpath.path {
            item = item.get_child_dir_mut(path)?;
        }
        let item = item.get_child_dir_mut(&last)?;
        assert!(item.is_dir());
        Ok(item)
    }

    /// Return the current path.
    pub fn pwd(&self) -> String {
        self.path.to_string()
    }

    /// Move to the child directory, just like "cd xxx/yyy".
    fn move_forward(&mut self, name: String) -> Result<()> {
        let mut path = self.path.clone();
        path.join_str(name);
        self.dir_item_at(&path.path)?;  // check if getting directory succeeds
        self.path = path;
        Ok(())
    }

    /// Move to the parent directory, just like "cd ../".
    fn move_backward(&mut self, level: usize) -> Result<()> {
        self.path.pops(level);
        Ok(())
    }

    /// Move to the home (root) directory.
    pub fn move_to_home(&mut self) {
        self.path = PathVector::new();
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
        let path = self.path.to_string();
        format!("/[{}]/{} > ", name, path)
    }
    
    /// Read the file content at `path`.
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
                    TreeError::new(format!("{}", err))
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

    /// Open file at `path` using default application.
    pub fn open_file(&self, path: &String) -> Result<()> {
        use open::that;
        let pathvec = self.resolve_virtual_path(path)?;
        let item = self.item_at(pathvec)?;
        let entity_path = match item.entity.as_ref() {
            Some(path) => path,
            None => return Err(TreeError::new("No entity".to_string())),
        };
        let path = match resolve_path(entity_path.to_str().unwrap()){
            Ok(path) => path,
            Err(err) => return Err(TreeError::new(format!("{}", err))),
        };

        match that(path) {
            Ok(_) => Ok(()),
            Err(err) => Err(
                TreeError::new(format!("Error opening file: {}", err))
            )
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
        let item = match self.dir_item_at_mut(&pathvec) {
            Ok(item) => item,
            Err(_) => self.current_item_mut()?,
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
        
        let item = self.dir_item_at_mut(&pathvec)?;
        item.remove_child(&file_name)
    }

    /// Return the text for "ls" command.
    pub fn ls_simple(&self, path: Option<String>) -> Result<String> {
        let path = match path {
            Some(path) => path,
            None => ".".to_string(),
        };
        let pathvec = self.resolve_virtual_path(&path)?;
        let item = self.dir_item_at(&pathvec)?;
        let children: Vec<String> = item.children_names();
        Ok(children.join(" "))
    }

    /// Return the text for "ls --desc" command.
    pub fn ls_detailed(&self, path: Option<String>) -> Result<String> {
        let path = match path {
            Some(path) => path,
            None => ".".to_string(),
        };
        let pathvec = self.resolve_virtual_path(&path)?;
        let item = self.dir_item_at(&pathvec)?;
        let mut name_vec: Vec<String> = Vec::new();
        let mut desc_vec: Vec<String> = Vec::new();
        for child in item.iter_children().into_iter() {
            name_vec.push(child.name.clone());
            let default_desc = "<no description>".to_string();
            let desc = child.desc.as_ref().unwrap_or(&default_desc);
            desc_vec.push(desc.clone());
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
                let filename = filepath
                    .file_name()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string();
                (pathvec, filename)
            }
        };

        let item = self.item_at_mut(dirvec)?;
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
            None => "".to_string(),
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
                    return Err(
                        TreeError::new(
                            format!("{} already exists.", path.to_str().unwrap())
                        )
                    )
                }
            };
            count += 1;
        };

        // create a hidden file
        match std::fs::File::create(&vpath) {
            Ok(_) => {}
            Err(err) => {
                return Err(
                    TreeError::new(
                        format!("{}: {}", vpath.to_str().unwrap(), err)
                    )
                )
            }
        };
        let item = self.item_at_mut(pathvec)?;
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
            Err(err) => Err(
                TreeError::new(
                    format!("Error calling command: {}", err)
                )
            )
        }
    }
}

/// Resolve input path string (must exist) and return a PathBuf with an absolute
/// path. Input string can be a relative path in the virtual directory or an 
/// existing absolute path.
fn resolve_path(path: &str) -> std::io::Result<PathBuf> {
    if path.starts_with(".") || path.starts_with("/") {
        let curdir = std::env::current_dir()?;
        let path = path
            .strip_prefix(".")
            .unwrap_or(path)
            .strip_prefix("/")
            .unwrap_or(path);
        let joined = std::path::Path::new(&curdir).join(path);
        Ok(joined)
    }
    else {
        let joined = std::path::Path::new(&path).to_path_buf();
        Ok(joined)
    }
}

// ---------------------------------------------------------------------
//   test
// ---------------------------------------------------------------------

#[cfg(test)]
mod test_tree_model {
    use super::*;

    // test
    //   ├─ dir-A
    //   │   └─ item.txt
    //   └─ dir-B
    const JSON_0: &str = r#"{
        "name": "test",
        "children": [
            {
                "name": "dir-A",
                "children": [
                    {
                        "name": "item.txt",
                        "children": [],
                        "desc": "test item",
                        "entity": "./src/main.rs"
                    }
                ],
                "desc": null,
                "entity": null
            },
            {
                "name": "dir-B",
                "children": [],
                "desc": null,
                "entity": null
            }
        ],
        "desc": null,
        "entity": null
    }"#;

    #[test]
    fn test_moving() {
        let mut tree = TreeModel::from_string(JSON_0);
        assert_eq!(tree.root.children_names(), vec!["dir-A", "dir-B"]);
        assert_eq!(tree.current_item().unwrap().name, "test");
        
        tree.move_forward("dir-A".to_string()).unwrap();
        assert_eq!(tree.current_item().unwrap().name, "dir-A");
        
        let result = tree.move_forward("item.txt".to_string());
        assert!(result.is_err());
        
        tree.move_backward(1).unwrap();
        assert_eq!(tree.current_item().unwrap().name, "test");
    }

    // implementation just for test
    impl TreeModel {
        fn assert_path_equal(&self, l: &str, path: &str) {
            let path1 = self.resolve_virtual_path(&l.to_string())
                .unwrap()
                .join("/");
            let path2 = path.to_string();
            assert_eq!(path1, path2);
        }
    }

    #[test]
    fn test_resolve() {
        let mut tree = TreeModel::from_string(JSON_0);
        tree.assert_path_equal(".", "");
        tree.assert_path_equal("./", "");
        tree.assert_path_equal("dir-A", "dir-A");
        tree.assert_path_equal("/dir-A", "dir-A");
        tree.assert_path_equal("./dir-A", "dir-A");

        tree.move_forward("dir-A".to_string()).unwrap();
        tree.assert_path_equal(".", "dir-A");
        tree.assert_path_equal("./item.txt", "dir-A/item.txt");
        tree.assert_path_equal("../", "");
        tree.assert_path_equal("../dir-B", "dir-B");

    }

    #[test]
    fn test_ls_default() {
        let tree = TreeModel::from_string(JSON_0);
        assert_eq!(tree.ls_simple(None).unwrap(), "dir-A dir-B");
    }
    
    #[test]
    fn test_ls_parametric() {
        let tree = TreeModel::from_string(JSON_0);
        assert_eq!(tree.ls_simple(Some("dir-A".to_string())).unwrap(), "item.txt");
    }
    

    #[test]
    fn test_mkdir() {
        let mut tree = TreeModel::from_string(JSON_0);
        let dirname = "dir-C".to_string();
        tree.make_directory(&dirname).unwrap();
        assert_eq!(tree.ls_simple(None).unwrap(), "dir-A dir-B dir-C");
    }

    // test
    //   ├─ NAME (file)
    //   └─ NAME
    //       └─ item.txt
    const JSON_1: &str = r#"{
        "name": "test",
        "children": [
            {
                "name": "NAME",
                "children": [],
                "desc": null,
                "entity": "./src/main.rs"
            },
            {
                "name": "NAME",
                "children": [
                    {
                        "name": "item.txt",
                        "children": [],
                        "desc": "test item",
                        "entity": "./src/main.rs"
                    }
                ],
                "desc": null,
                "entity": null
            }
        ],
        "desc": null,
        "entity": null
    }"#;

    #[test]
    fn test_duplicated_cd() {
        let mut tree = TreeModel::from_string(JSON_1);
        {
            let citem = tree.current_item().unwrap();
            let mut iter = citem.iter_children();
            let item0 = iter.next().unwrap();
            assert!(item0.is_file());
            assert!(item0.entity.is_some());
            let item1 = iter.next().unwrap();
            assert!(item1.is_dir());
            assert!(item1.entity.is_none());
        }
        tree.move_forward("NAME".to_string()).unwrap();
        {
            let item = tree.current_item().unwrap();
            assert!(item.is_dir());
            assert!(item.entity.is_none());
        }
        assert_eq!(tree.ls_simple(None).unwrap(), "item.txt");
    }
}
