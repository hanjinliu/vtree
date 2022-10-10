use std::cell::RefCell;

/// Parse a string into a vector of strings in a CLI-like manner.
/// # Example
/// ```
/// parse_string("cd dirname"); // returns ["cd", "dirname"]
/// parse_string("cd \"dir name\""); // returns ["cd", "dir name"]
/// ```
pub fn parse_string(str: &String) -> Vec<String> {
    let mut inputs: Vec<String> = Vec::new();
    let mut buf: Vec<char> = Vec::new();
    let stack: RefCell<Option<char>> = RefCell::new(None);
    for c in str.chars() {
        if c != ' ' {
            if c == '"' || c == '\''{
                let current_stack = *stack.borrow();
                match current_stack {
                    Some(stack_char) => {
                        if stack_char == c {
                            stack.replace(None);
                            inputs.push(buf.iter().collect());
                            buf.clear();
                        }
                        else {
                            buf.push(c);
                        }
                    },
                    None => {
                        stack.replace(Some(c));
                    }
                }
            }
            else {
                buf.push(c);
            }
        }
        else {
            // c == ' '
            let x = stack.borrow_mut();
            match *x {
                None => {
                    // if no quotation mark is stacked, then flush the buffer.
                    if buf.len() > 0 {
                        inputs.push(buf.iter().collect::<String>());
                        buf.clear();
                    }
                }
                Some(_) => {
                    // if quotation mark is stacked, then push the character to the buffer.
                    buf.push(' ');
                }
            }
        }
    }
    if buf.len() > 0 {
        inputs.push(buf.into_iter().collect::<String>().trim().to_string());
    }
    inputs
}

/// Parse a string into a vector of strings in a CLI-like manner.
/// # Example
/// ```
/// parse_string("cd dirname"); // returns ["cd", "dirname"]
/// parse_string("cd \"dir name\""); // returns ["cd", "dir name"]
/// ```
pub fn parse_string_raw(str: &String) -> Vec<String> {
    let mut inputs: Vec<String> = Vec::new();
    let mut buf: Vec<char> = Vec::new();
    let stack: RefCell<Option<char>> = RefCell::new(None);
    for c in str.chars() {
        if c != ' ' {
            buf.push(c);
            if c == '"' || c == '\''{
                let current_stack = *stack.borrow();
                match current_stack {
                    Some(stack_char) => {
                        if stack_char == c {
                            stack.replace(None);
                            inputs.push(buf.iter().collect());
                            buf.clear();
                        }
                    },
                    None => {
                        stack.replace(Some(c));
                    }
                }
            }
        }
        else {
            // c == ' '
            let x = stack.borrow_mut();
            match *x {
                None => {
                    // if no quotation mark is stacked, then flush the buffer.
                    if buf.len() > 0 {
                        inputs.push(buf.iter().collect::<String>());
                        buf.clear();
                    }
                    inputs.push(" ".to_string());
                }
                Some(_) => {
                    buf.push(' ');
                }
            }
        }
    }
    // flush the buffer
    if buf.len() > 0 {
        inputs.push(buf.into_iter().collect::<String>().to_string());
    }
    inputs
}

#[cfg(test)]
mod test_parse_string {
    use super::*;
    #[test]
    fn test_parse() {
        let val = parse_string(&"cd /usr/bin".to_string());
        assert_eq!(val, vec!["cd".to_string(), "/usr/bin".to_string()]);
    }

    #[test]
    fn test_parse_quoted() {
        let val = parse_string(&"cd \"dir name\"".to_string());
        assert_eq!(val, vec!["cd".to_string(), "dir name".to_string()]);
    }

    #[test]
    fn test_parse_quoted_and_not_quoted() {
        let val = parse_string(&"cd \"dir name\" dir name".to_string());
        assert_eq!(
            val,
            vec!["cd".to_string(), "dir name".to_string(), "dir".to_string(), "name".to_string()]
        );
    }
    
    #[test]
    fn test_parse_quote_not_closed() {
        let val = parse_string(&"cd \"dir name".to_string());
        assert_eq!(
            val,
            vec!["cd".to_string(), "dir name".to_string()]
        );
    }
}

#[cfg(test)]
mod test_parse_string_quoted {
    use super::*;

    #[test]
    fn test_parse() {
        let val = parse_string_raw(&"cd /usr/bin".to_string());
        assert_eq!(val, vec!["cd".to_string(), " ".to_string(), "/usr/bin".to_string()]);
    }

    #[test]
    fn test_parse_quoted() {
        let val = parse_string_raw(&"cd \"dir name\"".to_string());
        assert_eq!(val, vec!["cd".to_string(), " ".to_string(), "\"dir name\"".to_string()]);
    }

    #[test]
    fn test_parse_quoted_and_not_quoted() {
        let val = parse_string_raw(&"cd \"dir name\" dir name".to_string());
        assert_eq!(
            val,
            vec![
                "cd".to_string(), " ".to_string(), "\"dir name\"".to_string(), " ".to_string(),
                 "dir".to_string(), " ".to_string(), "name".to_string()]
        );
    }
    
    #[test]
    fn test_parse_quote_not_closed() {
        let val = parse_string_raw(&"cd \"dir name".to_string());
        assert_eq!(
            val,
            vec!["cd".to_string(), " ".to_string(), "\"dir name".to_string()]
        );
    }

    #[test]
    fn test_parse_starts_with_space() {
        let val = parse_string_raw(&" a b c".to_string());
        assert_eq!(
            val,
            vec![" ".to_string(), "a".to_string(), " ".to_string(), "b".to_string(), 
                 " ".to_string(), "c".to_string()]
        );
    }

    #[test]
    fn test_parse_ends_with_space() {
        let val = parse_string_raw(&"a b c ".to_string());
        assert_eq!(
            val,
            vec!["a".to_string(), " ".to_string(), "b".to_string(), " ".to_string(), 
                 "c".to_string(), " ".to_string()]
        );
    }

    #[test]
    fn test_parse_many_spaces() {
        let val = parse_string_raw(&"a  bb  c  ".to_string());
        assert_eq!(
            val,
            vec!["a".to_string(), " ".to_string(), " ".to_string(), "bb".to_string(), " ".to_string(), 
                 " ".to_string(), "c".to_string(), " ".to_string(), " ".to_string()]
        );
    }
    
}