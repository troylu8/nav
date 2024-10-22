
use std::env;
use std::io::Error;


use map::Map;

use crossterm::event::*;



fn main() -> Result<(), Error> {
    
    let mut map = Map::new(env::current_dir()?)?;

    map.print()?;

    loop {
        if let Event::Key(KeyEvent {kind: KeyEventKind::Press, code, ..}) = read()? {
            match code {
                KeyCode::Left => {
                    map.move_out()?;
                },
                KeyCode::Right => {
                    let _ = map.move_into();
                },
                KeyCode::Up => {
                    map.move_up()?;
                },
                KeyCode::Down => {
                    map.move_down()?;
                },
                _ => {}
            }
        }
    }

}