use std::io::Write;
use std::str::FromStr;

pub fn input() -> std::io::Result<String> {
    let mut val = String::new();
    std::io::stdin().read_line(&mut val)?;
    Ok(val)
}

/// Available input commands.
pub enum InputCommand {
    Cd,
    Tree,
    Ls,
    Mkdir,
    Rm,
    Pwd,
    Cat,
    Exit,
}

impl std::fmt::Display for InputCommand {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InputCommand::Cd   => write!(f, "cd"),
            InputCommand::Tree => write!(f, "tree"),
            InputCommand::Ls => write!(f, "ls"),
            InputCommand::Mkdir => write!(f, "mkdir"),
            InputCommand::Rm => write!(f, "rm"),
            InputCommand::Pwd => write!(f, "pwd"),
            InputCommand::Cat => write!(f, "cat"),
            InputCommand::Exit => write!(f, "exit"),
        }
    }
}

impl std::str::FromStr for InputCommand {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "cd"    => Ok(InputCommand::Cd),
            "tree"  => Ok(InputCommand::Tree),
            "ls"    => Ok(InputCommand::Ls),
            "pwd"   => Ok(InputCommand::Pwd),
            "cat"   => Ok(InputCommand::Cat),
            "mkdir" => Ok(InputCommand::Mkdir),
            "rm"    => Ok(InputCommand::Rm),
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
        let cmd = iter.next().unwrap().to_string();
        let mut args: Vec<String> = Vec::new();
        for arg in iter {
            args.push(arg.to_string());
        }
        let input_cmd = InputCommand::from_str(cmd.as_str()).unwrap();
        Ok(Input{cmd: input_cmd, args: args})
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
