
use std::env;
use std::fs;
use std::io::stdout;
use std::io::Error;

use crossterm::event::*;
use crossterm::{execute, cursor, style, terminal};

use map::Map;

fn main() -> Result<(), Error> {
    
    let map_home = env::var("MAP_HOME").unwrap();
    
    execute!(
        stdout(),
        crossterm::cursor::Hide,
        terminal::DisableLineWrap,
    )?;

    let mut map = Map::new(env::current_dir()?)?;

    map.print()?;

    loop {
        if let Event::Key(KeyEvent {kind: KeyEventKind::Press, code, ..}) = read()? {
            match code {
                KeyCode::Left => map.move_out()?,
                KeyCode::Right => { let _ = map.move_into(); },
                KeyCode::Up | KeyCode::BackTab => map.move_up()?,
                KeyCode::Down | KeyCode::Tab => map.move_down()?,
                KeyCode::Enter => {
                    fs::write(map_home + "\\output.txt", map.get_path().to_str().unwrap())?;
                    break;
                },
                KeyCode::Esc => {
                    fs::write(map_home + "\\output.txt", ".")?;
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