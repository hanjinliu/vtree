pub mod error;
pub mod tree;
pub mod terminal;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use structopt::StructOpt;
use terminal::{Input, InputCommand};
// use crate::error::VTreeError;


// The main command line interface for vtree.
#[derive(StructOpt)]
#[structopt(about = "Virtural file tree manager")]
enum VTree {
    Init,  // vtree init: initialize vtree meta directory.
    New {name: Option<String>},  // vtree new {name}: create a new virtual directory.
    Tree {name: String},  // vtree tree {name}: show the virtual directory tree.
    Enter {name: String},  // vtree enter {name}: enter the virtual directory.
    List,  // vtree list: show all the names of virtual root trees.
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
        let input = Input::from_input(&prefix)?;
        match input.cmd {
            InputCommand::Cd => {
                tree.move_by_string(&input.args[0]).unwrap();
            }
            InputCommand::Tree => {
                println!("{}", tree.current);
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
                tree.ls_simple(name).unwrap();
            }
            InputCommand::Pwd => {
                println!("./{}/{}", tree.root.name, tree.pwd());
            }
            InputCommand::Cat => {
                let arg = &input.args[0];
                tree.print_file(arg)?;
            }
            InputCommand::Touch => {
                let name = &input.args[0];
                if tree.current.has(&name) {
                    return Err(
                        std::io::Error::new(
                            std::io::ErrorKind::AlreadyExists,
                            format!("{} already exists.", name),
                        )
                    )
                }
                let mut vpath_cand = get_vtree_path(true)?
                    .join(_VIRTUAL_FILES)
                    .join(name);
                // find unique file name
                let vpath_copy = vpath_cand.clone();
                let stem = vpath_copy.file_stem().unwrap().to_str().unwrap();
                let ext = vpath_copy.extension().unwrap().to_str().unwrap();
                let mut count = 0;
                let vpath = loop {
                    if !vpath_cand.exists() {
                        break vpath_cand;
                    }
                    let filename = format!("{}-{}.{}", stem, count, ext);
                    vpath_cand = vpath_cand.parent().unwrap().join(filename);
                    count += 1;
                };
                File::create(&vpath)?;
                let mut item = tree.current.clone();
                item.add_item(&name, vpath).unwrap();
                tree.set_item_at(tree.path.path.clone(), item).unwrap();
            }
            InputCommand::Open => {
                tree.open_file(&input.args[0])?;
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
                tree.add_alias(dst, PathBuf::from(src)).unwrap();
            }
            InputCommand::Mkdir => {
                tree.mkdir(&input.args[0]).unwrap();
            }
            InputCommand::Rm => {
                tree.rm(&input.args[0]).unwrap();
            }
            InputCommand::Exit => {
                // TODO: save the tree to the json file.
                break;
            }
        }
    }
    Ok(())
}

fn list() -> std::io::Result<()> {
    let mut path = get_vtree_path(true)?;
    path.push(_TREES);
    // iterate all the json files and print each name and description.
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            let item = tree::TreeItem::from_file(&path)?;
            match item.desc {
                Some(value) => {println!("{}: {}", item.name, value);}
                None => {println!("{}", item.name);}
            }
        }
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
        VTree::List => {
            list().unwrap();
        }
    };
}
