#[derive(Debug)]
pub struct VTreeError {
    msg: String
}

impl VTreeError {
    pub fn new(msg: &str) -> Self {
        VTreeError{msg: msg.to_string()}
    }
}

impl std::fmt::Display for VTreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f,"{}",self.msg)
    }
}

impl std::error::Error for VTreeError {
    fn description(&self) -> &str {
        &self.msg
    }
}
