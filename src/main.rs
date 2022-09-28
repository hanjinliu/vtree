pub mod error;
pub mod tree;
use std::fs::File;
use std::io::Write;
use structopt::StructOpt;
// use crate::error::VTreeError;


// The main command line interface for vtree.
#[derive(StructOpt)]
#[structopt(about = "Virtural file tree manager")]
enum VTree {
    Init,  // vtree init: initialize vtree meta directory.
    New {name: Option<String>}  // vtree new {name}: create a new virtual directory.
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
    };
}
