use std::io;
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::path::PathBuf;
use super::super::terminal::VCommand;
use super::super::tree;
use super::super::{get_json_path, get_relative_vtree_path};
use super::vtui::{process_keys, App};

const _VIRTUAL_FILES: &str = "virtual-files";

pub fn enter(name: String) -> std::io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let root = get_json_path(&name)?;
    if !root.exists() {
        return Err(
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Virtual directory {} does not exist.", name),
            )
        )?;
    }
    let tree = tree::TreeModel::from_file(&root)?;
    let mut app = App::new(tree);

    loop {
        // get valid input
        let user_input = process_keys(&mut terminal, &mut app)?;

        let input = match VCommand::from_string(&user_input){
            Ok(input) => input,
            Err(e) => {
                app.print_prefix(format!("{}", e), true);
                continue;
            }
        };
        let output = match input {
            VCommand::Empty => {
                Ok(())
            }
            VCommand::Cd { name } => {
                match name {
                    Some(path) => {
                        app.tree.move_by_string(&path)
                    }
                    None => {
                        app.tree.move_to_home()
                    }
                }
            
            }
            VCommand::Tree { name } => {
                match name {
                    Some(name) => {
                        let str = match app.tree.current.get_offspring(&name) {
                            Ok(item) => {
                                format!("{}", item)
                            }
                            Err(e) => {
                                format!("{}", e)
                            }
                        };
                        app.print_prefix(str, true);
                    }
                    None => {
                        app.print_prefix(format!("{}", app.tree.current), true);
                    }
                }
                Ok(())
            }
            VCommand::Ls { name, desc } => {
                let str = if desc {
                    app.tree.ls_with_desc(name)
                }
                else {
                    app.tree.ls_simple(name)
                };
                match str {
                    Ok(s) => {
                        app.print_prefix(format!("{}", s), true);
                        Ok(())
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            VCommand::Pwd => {
                app.print_prefix(format!("./{}/{}", app.tree.root.name, app.tree.pwd()), true);
                Ok(())
            }
            VCommand::Cat { name } => {
                app.tree.print_file(&name)
            }
            VCommand::Touch { name } => {
                let vpath_cand = get_relative_vtree_path(true)?
                    .join(_VIRTUAL_FILES)
                    .join(name.clone());
                // find unique file name
                app.tree.create_new_file(&name, vpath_cand)
            }
            VCommand::Open { name } => {
                app.tree.open_file(&name)
            }
            VCommand::Cp { src, dst } => {
                app.tree.add_alias(dst, PathBuf::from(src))
            }
            VCommand::Desc { name, desc } => {
                let mut item = match name {
                    Some(name) => {
                        match app.tree.current.get_child_mut(&name){
                            Ok(item) => item,
                            Err(e) => {
                                println!("{}", e);
                                continue;
                            },
                        }
                    }
                    None => {
                        &mut app.tree.current
                    }
                };
                match desc {
                    Some(desc) => {
                        // let mut item = item.clone();
                        item.desc = Some(desc);
                    }
                    None => {
                        // app.print_prefix(format!("{}", item.desc.as_ref().unwrap_or(&"".to_string())));
                    }
                }
                Ok(())
            }
            VCommand::Call { vec } => {
                app.tree.call_command(&vec)
            }
            VCommand::Mkdir { name } => {
                app.tree.mkdir(&name)
            }
            VCommand::Rm { name } => {
                match app.tree.current.get_child(&name) {
                    Ok(item) => {
                        match &item.entity {
                            Some(path) => {
                                std::fs::remove_file(path)?;
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        app.print_prefix(format!("{}", err), true);
                        continue;
                    }
                };
                app.tree.rm(&name)
            }
            VCommand::Exit => {
                app.tree.to_file(root.as_path())?;
                break;
            }
        };
        match output {
            Ok(_) => {}
            Err(e) => {
                app.print_prefix(format!("{}", e), true);
            }
        }
    }
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
