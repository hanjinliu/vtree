use std::io::Write;
use structopt::StructOpt;
use super::parser::parse_string;

pub fn input() -> std::io::Result<String> {
    let mut val = String::new();
    std::io::stdin().read_line(&mut val)?;
    Ok(val)
}

// The virtual command line interface.
#[derive(StructOpt)]
#[structopt()]
pub enum VCommand {
    Call {vec: Vec<String>},
    Cat {
        name: String,
        #[structopt(short="n", long="number", about="Show line number")]
        number: bool,
    },
    Cd {name: Option<String>},
    Cp {src: String, dst: Option<String>},
    Desc {
        name: Option<String>, 
        #[structopt(short="d", long="desc", about = "Descriptions")]
        desc: Option<String>,
    },
    Empty,
    Exit {
        #[structopt(long="discard", about="Discard changes and exit")]
        discard: bool,
    },
    Ls {
        name: Option<String>,
        #[structopt(short="d", long="desc", about="Show descriptions")]
        desc: bool,
    },
    Mkdir {name: String},
    Mv {src: String, dst: String},
    Open {name: String},
    Pwd,
    Rm {name: String},
    Touch {name: String},
    Tree {name: Option<String>},
}

impl VCommand {
    pub fn from_string(val: &String) -> std::result::Result<Self, structopt::clap::Error> {
        if val.trim().len() == 0 {
            return Ok(VCommand::Empty);
        }
        let mut args = vec![String::from("virtual-command")];
        for arg in parse_string(val) {
            if arg.len() > 0 {
                args.push(arg);
            }
        }
        Self::from_iter_safe(&args)
    }

    pub fn from_line(prefix: &String) -> std::result::Result<Self, structopt::clap::Error> {
        print!("{}", prefix);
        let _ = std::io::stdout().flush();
        let val = input()?;
        Self::from_string(&val)
    }
}
