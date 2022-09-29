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
        let tree = tree::TreeItem::from_file(&path)?;
        println!("{}", tree);
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
    let mut system = tree::TreeSystem::from_file(&root)?;

    loop {
        let prefix = system.as_prefix();
        let input = Input::from_input(&prefix)?;
        match input.cmd {
            InputCommand::Cd => {
                system.move_by_string(&input.args[0]).unwrap();
            }
            InputCommand::Tree => {
                println!("{}", system.current);
            }
            InputCommand::Ls => {
                let tree = system.current.clone();
                let children: Vec<String> = tree.into_iter().map(|item| item.name).collect();
                println!("{}", children.join(" "));
            }
            InputCommand::Pwd => {
                println!("{}", system.pwd());
            }
            InputCommand::Mkdir => {
                let mut root = system.root.clone();
                let mut tree = system.current.clone();
                tree.mkdir(&name).unwrap();
                root.update_child(&name, tree).unwrap();
            }
            InputCommand::Rm => {
                let mut root = system.root.clone();
                let mut tree = system.current.clone();
                tree.rm(&name).unwrap();
                root.update_child(&name, tree).unwrap();
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
