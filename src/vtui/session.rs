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
use super::{
    vtui::process_keys, 
    app::App,
};

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
                app.print_error(e);
                continue;
            }
        };
        let output = match input {
            VCommand::Empty => {
                Ok(())
            }
            VCommand::Cd { name } => {
                match name {
                    Some(path) => app.tree.move_by_string(&path),
                    None => Ok(app.tree.move_to_home()),
                }
            
            }
            VCommand::Tree { name } => {
                match name {
                    Some(name) => {
                        match app.tree.get_item(&name) {
                            Ok(item) => {
                                app.print_text(format!("{}", item))
                            }
                            Err(e) => {
                                app.print_error(e);
                            }
                        };
                    }
                    None => {
                        match app.tree.current_item() {
                            Ok(item) => {
                                app.print_text(format!("{}", item))
                            }
                            Err(e) => {
                                app.print_error(e);
                            }
                        };
                    }
                }
                Ok(())
            }
            VCommand::Ls { name, desc } => {
                let str = if desc {
                    app.tree.ls_with_desc(name)
                } else {
                    app.tree.ls_simple(name)
                };
                match str {
                    Ok(s) => {
                        app.print_text(s);
                        Ok(())
                    }
                    Err(e) => {
                        Err(e)
                    }
                }
            }
            VCommand::Pwd => {
                app.print_text(format!("./{}/{}", app.tree.root.name, app.tree.pwd()));
                Ok(())
            }
            VCommand::Cat { name } => {
                match app.tree.read_file(&name) {
                    Ok(text) => app.print_text(text),
                    Err(e) => app.print_error(e),
                };
                Ok(())
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
                app.tree.add_alias(dst.as_ref(), PathBuf::from(src))
            }
            VCommand::Desc { name, desc } => {
                let item_result = match name {
                    Some(name) => app.tree.get_item(&name),
                    None => app.tree.get_item(&app.tree.pwd()),
                };
                let mut item = item_result.unwrap().clone();
                let item = item.as_mut();
                match desc {
                    Some(desc) => {
                        item.desc = Some(desc);
                    }
                    None => {
                        // TODO: enter description mode
                    }
                }
                Ok(())
            }
            VCommand::Call { vec } => {
                terminal.show_cursor()?;
                app.tree.call_command(&vec)
            }
            VCommand::Mkdir { name } => {
                app.tree.make_directory(&name)
            }
            VCommand::Rm { name } => {
                match app.tree.get_item(&name) {
                    Ok(item) => {
                        match &item.entity {
                            Some(path) => {
                                let vfiles_path = get_relative_vtree_path(true)?
                                    .join(_VIRTUAL_FILES);
                                if path.starts_with(vfiles_path) {
                                    std::fs::remove_file(path)?;
                                }
                            }
                            None => {}
                        }
                    }
                    Err(err) => {
                        app.print_error(err);
                        continue;
                    }
                };
                app.tree.remove_child(&name)
            }
            VCommand::Exit { discard } => {
                if !discard {
                    app.tree.to_file(root.as_path())?;
                }
                break;
            }
        };
        match output {
            Ok(_) => {}
            Err(err) => {
                app.print_error(err);
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
