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
    let prefix = format!("[{}] > ", name);
    loop {
        let input = Input::from_input(&prefix)?;
        match input.cmd {
            InputCommand::Cd => {
                let path = get_json_path(&name)?;
                if path.exists() {
                    // TODO: change the current directory
                }
            }
            InputCommand::Tree => {
                let path = get_json_path(&name)?;
                if path.exists() {
                    let tree = tree::TreeItem::from_file(&path)?;
                    println!("{}", tree);
                }
            }
            InputCommand::Ls => {
                let path = get_json_path(&name)?;
                if path.exists() {
                    let tree = tree::TreeItem::from_file(&path)?;
                    let children: Vec<String> = tree.into_iter().map(|item| item.name).collect();
                    println!("{}", children.join(" "));
                }
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
