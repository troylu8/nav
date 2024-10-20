
use std::env;
use std::io::Error;


use map::Map;



fn main() -> Result<(), Error> {
    
    let mut map = Map::new(env::current_dir()?)?;

    map.print()?;

    // dbg!(map);

    // let desired_x = path_buf.iter().fold(0, |acc, x| acc + x.len() + 3) as u16 // include " \ " after every component
    //     - 1 // " >" is behind the last component instead of " \ "
    //     - 1 // 0 indexed
    //     - 4; // ignore "\" + " \ " used by root dir

    // let x = ;

    // // print_path(&path_buf, x, 0);
    // let mut file_lib= FileLibrary::new(&path_buf, height as usize);
    // a.print(x + 4);
    // print_path(&path_buf, x, y);

    // loop {
    //     if let  Event::Key(KeyEvent {code, ..}) = event::read()? {
    //         match code {
    //             KeyCode::Left => {
    //                 path_buf.pop();
    //             },
    //             KeyCode::Right => {
    //                 path_buf.push(file_lib.curr_ls.ls[file_lib.curr_ls.curr_i].file_name());
    //             },
    //             KeyCode::Up => {
    //                 file_lib.curr_ls.curr_i -= 1;
                    
    //             },
    //             KeyCode::Down => {
    //                 file_lib.curr_ls.curr_i += 1;
    //             },
    //             _ => { continue; } // of pressed key wasn't an arrow key, don't set listing again
    //         }
    //         file_lib.set_listing(&path_buf);

    //         clear();
    //         file_lib.print(x);
    //     }
    // }

    Ok(())
}