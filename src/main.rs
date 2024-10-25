
use std::env;
use std::fs;
use std::io::stdout;
use std::io::Error;

use crossterm::event::*;
use crossterm::{execute, cursor, style, terminal};

use map::Map;

fn main() -> Result<(), Error> {
    
    let nav_home = env::var("NAV_HOME").unwrap();
    
    execute!(
        stdout(),
        crossterm::cursor::Hide,
        terminal::DisableLineWrap,
    )?;

    let mut map = Map::new(env::current_dir()?)?;

    loop {
        
        if let Event::Key( KeyEvent {kind: KeyEventKind::Press, code, modifiers, ..}) = read()? {
            match code {
                KeyCode::Left => map.move_out()?,
                KeyCode::Right => { 
                    match map.move_into() {
                        // ignore permission denied error
                        Err(err) if err.kind() != std::io::ErrorKind::PermissionDenied => return Err(err),
                        _ => {}
                    }
                }, 
                KeyCode::Up | KeyCode::BackTab => map.move_up()?,
                KeyCode::Down | KeyCode::Tab => map.move_down()?,
                KeyCode::Enter => {
                    fs::write(nav_home + "\\map\\map_dest.txt", map.get_path().to_str().unwrap())?;
                    break;
                },
                KeyCode::Esc => {
                    fs::write(nav_home + "\\map\\map_dest.txt", ".")?;
                    break;
                }
                KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => {
                    fs::write(nav_home + "\\map\\map_dest.txt", ".")?;
                    break;
                }
                _ => {}
            }
        }
    }

    execute!( stdout(), cursor::Show )?;
    execute!( stdout(), style::SetAttribute(style::Attribute::Reset) )?;

    Ok(())

}