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
    Test {
        name: Option<String>,
    }
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

/// Enter a virtual tree and launch vtree session.
/// Commands such as "cd" and "ls" are replaced with virtual ones. Until "exit"
/// is called, you'll be in the virtual file system.
// fn enter(name: String) -> std::io::Result<()> {
//     let root = get_json_path(&name)?;
//     if !root.exists() {
//         return Err(
//             std::io::Error::new(
//                 std::io::ErrorKind::NotFound,
//                 format!("Virtual directory {} does not exist.", name),
//             )
//         )?;
//     }
//     let mut tree = tree::TreeModel::from_file(&root)?;

//     loop {
//         let prefix = tree.as_prefix();
//         // get valid input
//         let input = match VCommand::from_line(&prefix){
//             Ok(input) => input,
//             Err(e) => {
//                 println!("{}", e);
//                 continue;
//             }
//         };
//         let output = match input {
//             VCommand::Empty => {
//                 Ok(())
//             }
//             VCommand::Cd { name } => {
//                 match name {
//                     Some(path) => {
//                         tree.move_by_string(&path)
//                     }
//                     None => {
//                         tree.move_to_home()
//                     }
//                 }
            
//             }
//             VCommand::Tree { name } => {
//                 match name {
//                     Some(name) => {
//                         match tree.current.get_offspring(&name){
//                             Ok(item) => {
//                                 println!("{}", item)
//                             }
//                             Err(e) => {
//                                 println!("{}", e)
//                             }
//                         }
//                     }
//                     None => {
//                         println!("{}", tree.current);
//                     }
//                 }
//                 Ok(())
//             }
//             VCommand::Ls { name, desc } => {
//                 let str = if desc {
//                     tree.ls_with_desc(name)
//                 }
//                 else {
//                     tree.ls_simple(name)
//                 };
//                 match str {
//                     Ok(s) => {
//                         println!("{}", s);
//                         Ok(())
//                     }
//                     Err(e) => {
//                         Err(e)
//                     }
//                 }
//             }
//             VCommand::Pwd => {
//                 println!("./{}/{}", tree.root.name, tree.pwd());
//                 Ok(())
//             }
//             VCommand::Cat { name } => {
//                 tree.print_file(&name)
//             }
//             VCommand::Touch { name } => {
//                 let vpath_cand = get_relative_vtree_path(true)?
//                     .join(_VIRTUAL_FILES)
//                     .join(name.clone());
//                 // find unique file name
//                 tree.create_new_file(&name, vpath_cand)
//             }
//             VCommand::Open { name } => {
//                 tree.open_file(&name)
//             }
//             VCommand::Cp { src, dst } => {
//                 tree.add_alias(dst, PathBuf::from(src))
//             }
//             VCommand::Desc { name, desc } => {
//                 let mut item = match name {
//                     Some(name) => {
//                         match tree.current.get_child_mut(&name){
//                             Ok(item) => item,
//                             Err(e) => {
//                                 println!("{}", e);
//                                 continue;
//                             },
//                         }
//                     }
//                     None => {
//                         &mut tree.current
//                     }
//                 };
//                 match desc {
//                     Some(desc) => {
//                         // let mut item = item.clone();
//                         item.desc = Some(desc);
//                     }
//                     None => {
//                         println!("{}", item.desc.as_ref().unwrap_or(&"".to_string()));
//                     }
//                 }
//                 Ok(())
//             }
//             VCommand::Call { vec } => {
//                 tree.call_command(&vec)
//             }
//             VCommand::Mkdir { name } => {
//                 tree.mkdir(&name)
//             }
//             VCommand::Rm { name } => {
//                 match tree.current.get_child(&name) {
//                     Ok(item) => {
//                         match &item.entity {
//                             Some(path) => {
//                                 std::fs::remove_file(path)?;
//                             }
//                             None => {}
//                         }
//                     }
//                     Err(err) => {
//                         println!("{}", err);
//                         continue;
//                     }
//                 };
//                 tree.rm(&name)
//             }
//             VCommand::Exit => {
//                 tree.to_file(root.as_path())?;
//                 break;
//             }
//         };
//         match output {
//             Ok(_) => {}
//             Err(e) => {
//                 println!("{}", e);
//             }
//         }
//     }
//     Ok(())
// }


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
        VTree::Test { name } => {
            // run().unwrap();
        }
    };
}
