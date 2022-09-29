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
}

/// Check .vtree directory and search for virtual tree model stored in it.
/// # Errors
/// If the .vtree directory does not exist, return an error.
fn get_json_path(name: &String) -> std::io::Result<PathBuf> {
    let mut path = std::env::current_dir()?;
    path.push(".vtree");
    if !path.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "vtree is not initialized. Please run `vtree init` first.",
            )
        );
    }
    path.push(format!("{}.json", name));
    Ok(path)
}

/// Resolve input path string and return a PathBuf with an absolute path.
fn resolve_path(path: &String) -> std::io::Result<PathBuf> {
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

fn init() -> std::io::Result<()>{
    let mut path = std::env::current_dir()?;
    path.push(".vtree");

    if !path.exists() {
        // create a vtree hidden directory as /.vtree if it does not
        // exist.
        std::fs::create_dir(path.clone())?;
    }
    println!("Initialized vtree at {}", path.to_str().unwrap());
    Ok(())
}

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

fn tree(name: String) -> std::io::Result<()> {
    let path = get_json_path(&name)?;
    if path.exists() {
        let item = tree::TreeItem::from_file(&path)?;
        println!("{}", item);
    }
    Ok(())
}

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
                let item = tree.current.clone();
                let children: Vec<String> = item.into_iter().map(|item| item.name).collect();
                println!("{}", children.join(" "));
            }
            InputCommand::Pwd => {
                println!("./{}/{}", tree.root.name, tree.pwd());
            }
            InputCommand::Cat => {
                let arg = &input.args[0];
                let item = tree.current.get_child(&arg).unwrap();
                if item.is_file() {
                    let path = resolve_path(&item.name).unwrap();
                    let contents = std::fs::read_to_string(&path);
                    match contents {
                        Ok(value) => { println!("{}", value); }
                        Err(err) => { return Err(err); }
                    }
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
            InputCommand::Mkdir => {
                let mut item = tree.current.clone();
                item.mkdir(&input.args[0]).unwrap();
                tree.set_item_at(tree.path.path.clone(), item).unwrap();
            }
            InputCommand::Rm => {
                let mut item = tree.current.clone();
                item.rm(&input.args[0]).unwrap();
                tree.set_item_at(tree.path.path.clone(), item).unwrap();
            }
            InputCommand::Exit => {
                break;
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
    };
}
