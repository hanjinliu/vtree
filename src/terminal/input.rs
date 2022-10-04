use std::io::Write;
use std::str::FromStr;

pub fn input() -> std::io::Result<String> {
    let mut val = String::new();
    std::io::stdin().read_line(&mut val)?;
    Ok(val)
}

/// Available input commands.
pub enum InputCommand {
    NoCommand,
    Cd,
    Tree,
    Ls,
    Mkdir,
    Rm,
    Pwd,
    Cat,
    Touch,
    Open,
    Cp,
    Desc,
    Exit,
}

impl std::fmt::Display for InputCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InputCommand::NoCommand => write!(f, ""),
            InputCommand::Cd   => write!(f, "cd"),
            InputCommand::Tree => write!(f, "tree"),
            InputCommand::Ls => write!(f, "ls"),
            InputCommand::Mkdir => write!(f, "mkdir"),
            InputCommand::Rm => write!(f, "rm"),
            InputCommand::Pwd => write!(f, "pwd"),
            InputCommand::Cat => write!(f, "cat"),
            InputCommand::Touch => write!(f, "touch"),
            InputCommand::Open => write!(f, "open"),
            InputCommand::Cp => write!(f, "cp"),
            InputCommand::Desc => write!(f, "desc"),
            InputCommand::Exit => write!(f, "exit"),
        }
    }
}

impl std::str::FromStr for InputCommand {
    type Err = ();

    /// Convert a string to an input command. This function is used to parse the input 
    /// commands.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            ""      => Ok(InputCommand::NoCommand),
            "cd"    => Ok(InputCommand::Cd),
            "tree"  => Ok(InputCommand::Tree),
            "ls"    => Ok(InputCommand::Ls),
            "pwd"   => Ok(InputCommand::Pwd),
            "cat"   => Ok(InputCommand::Cat),
            "mkdir" => Ok(InputCommand::Mkdir),
            "rm"    => Ok(InputCommand::Rm),
            "touch" => Ok(InputCommand::Touch),
            "open"  => Ok(InputCommand::Open),
            "cp"    => Ok(InputCommand::Cp),
            "desc"  => Ok(InputCommand::Desc),
            "exit"  => Ok(InputCommand::Exit),
            _       => Err(()),
        }
    }
}

/// The structured input string.
pub struct Input {
    pub cmd: InputCommand,
    pub args: Vec<String>,
}

impl Input {
    /// Create a new input from the command line.
    pub fn from_input(prefix: &String) -> std::io::Result<Self> {
        print!("{}", prefix);
        let _ = std::io::stdout().flush();
        let val = input()?;
        let mut iter = val.split_whitespace();
        let cmd = iter.next().unwrap_or("").to_string();
        let mut args: Vec<String> = Vec::new();
        for arg in iter {
            args.push(arg.to_string());
        }
        match InputCommand::from_str(cmd.as_str()) {
            Ok(cmd) => Ok(Input{cmd: cmd, args: args}),
            Err(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                format!("Invalid command: {}", cmd),
            )),
        }
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.cmd)?;
        for arg in &self.args {
            write!(f, " {}", arg)?;
        }
        Ok(())
    }
}
