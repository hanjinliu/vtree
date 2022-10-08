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
    Empty,
    Cd {name: Option<String>},
    Tree {name: Option<String>},
    Ls {
        name: Option<String>,
        #[structopt(short="d", long="desc", about="Show descriptions")]
        desc: bool,
    },
    Mkdir {name: String},
    Rm {name: String},
    Pwd,
    Cat {name: String},
    Touch {name: String},
    Open {name: String},
    Cp {src: String, dst: Option<String>},
    Call {vec: Vec<String>},
    Desc {
        name: Option<String>, 
        #[structopt(short="d", long="desc", about = "Descriptions")]
        desc: Option<String>,
    },
    Exit,
}

impl VCommand {
    pub fn from_line(prefix: &String) -> std::result::Result<Self, structopt::clap::Error> {
        print!("{}", prefix);
        let _ = std::io::stdout().flush();
        let val = input()?;
        if val.trim().len() == 0 {
            return Ok(VCommand::Empty);
        }
        let mut args = vec![String::from("virtual-command")];
        for arg in parse_string(&val) {
            if arg.len() > 0 {
                args.push(arg);
            }
        }
        VCommand::from_iter_safe(&args)
    }
}
