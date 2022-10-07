pub mod error;
pub mod tree;
pub mod terminal;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use terminal::{Input, InputCommand};
use tree::TreeItem;
// use crate::error::VTreeError;


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
fn get_vtree_path(check: bool) -> std::io::Result<PathBuf> {
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

fn get_relative_vtree_path(check: bool) -> std::io::Result<PathBuf> {
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
fn get_json_path(name: &String) -> std::io::Result<PathBuf> {
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

/// Enter a virtual tree and launch vtree session.
/// Commands such as "cd" and "ls" are replaced with virtual ones. Until "exit"
/// is called, you'll be in the virtual file system.
fn enter(name: String) -> std::io::Result<()> {
    let root = get_json_path(&name)?;
    if !root.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Virtual directory {} does not exist.", name),
            )
        )?;
    }
    let mut tree = tree::TreeModel::from_file(&root)?;

    loop {
        let prefix = tree.as_prefix();
        // get valid input
        let input = match Input::from_input(&prefix){
            Ok(input) => input,
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };
        let output = match input.cmd {
            InputCommand::NoCommand => {
                Ok(())
            }
            InputCommand::Cd => {
                tree.move_by_string(&input.args[0])
            }
            InputCommand::Tree => {
                println!("{}", tree.current);
                Ok(())
            }
            InputCommand::Ls => {
                let name = {
                    if input.args.len() == 0 {
                        None
                    }
                    else {
                        Some(&input.args[0])
                    }
                };
                tree.ls_simple(name)
            }
            InputCommand::Pwd => {
                println!("./{}/{}", tree.root.name, tree.pwd());
                Ok(())
            }
            InputCommand::Cat => {
                let arg = &input.args[0];
                tree.print_file(arg)
            }
            InputCommand::Touch => {
                let name = &input.args[0];
                let vpath_cand = get_relative_vtree_path(true)?
                    .join(_VIRTUAL_FILES)
                    .join(name);
                // find unique file name
                tree.create_new_file(name, vpath_cand)
            }
            InputCommand::Open => {
                tree.open_file(&input.args[0])
            }
            InputCommand::Cp => {
                let src = &input.args[0];
                let dst = {
                    if input.args.len() == 1 {
                        None
                    }
                    else {
                        Some(&input.args[1])
                    }
                };
                tree.add_alias(dst, PathBuf::from(src))
            }
            InputCommand::Desc => {
                let item = tree.current.clone();  // TODO: avoid cloning
                match input.args.get(0) {
                    Some(desc) => {
                        tree.set_desc(desc).unwrap();
                    }
                    None => {
                        match item.desc {
                            Some(desc) => println!("{}", desc),
                            None => println!("No description."),
                        }
                    }
                }
                Ok(())
            }
            InputCommand::Call => {
                tree.call_command(&input.args)
            }
            InputCommand::Mkdir => {
                tree.mkdir(&input.args[0])
            }
            InputCommand::Rm => {
                let name = &input.args[0];
                match tree.current.get_child(name) {
                    Ok(item) => {
                        match &item.entity {
                            Some(path) => {
                                std::fs::remove_file(path)?;
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        println!("{}", err);
                        continue;
                    }
                };
                tree.rm(name)
            }
            InputCommand::Exit => {
                tree.to_file(root.as_path())?;
                break;
            }
        };
        match output {
            Ok(_) => {}
            Err(e) => {
                println!("{}", e);
            }
        }
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
