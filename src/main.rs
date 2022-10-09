pub mod tree;
pub mod terminal;
pub mod vtui;
use std::{fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;
use tree::TreeItem;
use vtui::enter;


// The main command line interface for vtree.
#[derive(StructOpt)]
#[structopt(about = "Virtural file tree manager")]
enum VTree {
    Init,  // vtree init: initialize vtree meta directory.
    New {name: Option<String>},  // vtree new {name}: create a new virtual directory.
    Tree {name: String},  // vtree tree {name}: show the virtual directory tree.
    Enter {name: String},  // vtree enter {name}: enter the virtual directory.
    List {contains: Option<String>},  // vtree list: show all the names of virtual root trees.
    Remove {
        name: String,
        #[structopt(long)]
        dry: bool,
    },  // vtree remove {name}: remove a virtual root tree.
}

// Subdirectory names used in vtree
const _VTREE: &str = ".vtree";
const _TREES: &str = "trees";
const _VIRTUAL_FILES: &str = "virtual-files";

/// Return the directory that .vtree directory should exists.
pub fn get_vtree_path(check: bool) -> std::io::Result<PathBuf> {
    let path = std::env::current_dir()?.join(_VTREE);
    if check && !path.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "vtree is not initialized. Please run `vtree init` first.",
            )
        );
    }
    Ok(path)
}

pub fn get_relative_vtree_path(check: bool) -> std::io::Result<PathBuf> {
    let path = std::env::current_dir()?.join(_VTREE);
    if check && !path.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "vtree is not initialized. Please run `vtree init` first.",
            )
        );
    }
    Ok(std::path::PathBuf::from(".").join(_VTREE))
}

/// Check .vtree directory and search for virtual tree model stored in it.
/// # Errors
/// If the .vtree directory does not exist, return an error.
pub fn get_json_path(name: &String) -> std::io::Result<PathBuf> {
    let path = get_vtree_path(true)?;
    Ok(path.join(_TREES).join(format!("{}.json", name)))
}

/// Initialize current directory with vtree metadata.
/// This command is the first one to run before using vtree.
fn init() -> std::io::Result<()>{
    let path = get_vtree_path(false)?;

    if !path.exists() {
        // create a vtree hidden directory as /.vtree if it does not
        // exist.
        std::fs::create_dir(path.clone())?;
    }

    for dir in [_TREES, _VIRTUAL_FILES] {
        let subdir = path.join(dir);
        if !subdir.exists() {
            std::fs::create_dir(subdir)?;
        }
    }

    println!("Initialized vtree at {}", path.to_str().unwrap());
    Ok(())
}

/// Create a new virtual root tree in the vtree meta directory.
/// # Errors
/// If .vtree directory does not exist, return an error.
fn new(name: String) -> std::io::Result<()> {
    let path = get_json_path(&name)?;
    if !path.exists() {
        // create a new virtual directory info as /.vtree/{name} if it does
        // not exist.
        let file = File::create(path)?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write(
            format!("{{\"name\": \"{}\", \"children\": []}}", name).as_bytes()
        )?;
        writer.flush()?;
    }
    Ok(())
}

/// Print out all the content under the virtual tree with the given name.
/// # Errors
/// If .vtree directory does not exist, return an error.
fn tree(name: String) -> std::io::Result<()> {
    let path = get_json_path(&name)?;
    if path.exists() {
        let item = tree::TreeItem::from_file(&path)?;
        println!("{}", item);
    }
    Ok(())
}

fn list(contains: Option<String>) -> std::io::Result<()> {
    let mut path = get_vtree_path(true)?;
    let contains = match contains {
        Some(s) => s,
        None => "".to_string(),
    };
    path.push(_TREES);
    // iterate all the json files and print each name and description.
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if contains == "" || path.file_name().unwrap().to_str().unwrap().contains(&contains){
                let item = tree::TreeItem::from_file(&path)?;
                match item.desc {
                    Some(value) => {println!("{}: {}", item.name, value);}
                    None => {println!("{}", item.name);}
                }
            }
        }
    }
    Ok(())
}

fn remove(name: String, dry: bool) -> std::io::Result<()> {
    let path = get_json_path(&name)?;
    if !path.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Virtual directory {} does not exist.", name),
            )
        );
    }
    // let tree = tree::TreeModel::from_file(&path)?;
    let tree = TreeItem::from_file(&path)?;
    let default = std::path::Path::new("");

    for item in &tree.values() {
        if let Some(path) = &item.entity {
            if path.parent().unwrap_or(default).ends_with(_VIRTUAL_FILES) {
                if dry {
                    println!("Remove: {}", path.display());
                }
                else {
                    std::fs::remove_file(path).unwrap_or(());
                }
            }
        }
    };
    
    if dry {
        println!("Remove: {}", path.display());
    }
    else {
        std::fs::remove_file(path)?;
    }
    
    Ok(())
}


fn main() {
    let args = VTree::from_args();
    match args {
        VTree::Init => {
            init().unwrap();
        }
        VTree::New { name } => {
            match name {
                Some(value) => {new(value).unwrap();}
                None => {new("default".to_string()).unwrap();}
            };
        }
        VTree::Tree { name } => {
            tree(name).unwrap();
        }
        VTree::Enter { name } => {
            enter(name).unwrap();
        }
        VTree::List { contains } => {
            list(contains).unwrap();
        }
        VTree::Remove { name, dry } => {
            remove(name, dry).unwrap();
        }
    };
}
