use std::fmt;

pub type Result<T> = std::result::Result<T, TreeError>;

#[derive(Debug, Clone)]
pub struct TreeError {
    msg: String,
}

impl TreeError {
    pub fn new(msg: String) -> Self {
        TreeError{msg}
    }
}

impl fmt::Display for TreeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for TreeError {
    
}