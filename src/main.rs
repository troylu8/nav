
use std::fmt::Display;
use std::fs::DirEntry;
use std::path::{Component, PathBuf};
use std::{env, fs};
use std::io::{stdout, Error};

use crossterm::*;
use crossterm::style::Print;

fn main() {
    
    let path_buf = env::current_dir().unwrap();

    let (width, height) = terminal::size().unwrap();
    let y = height / 2;

    
    // clear();


    let desired_x = path_buf.iter().fold(0, |acc, x| acc + x.len() + 3) as u16 // include " \ " after every component
        - 1 // " >" is behind the last component instead of " \ "
        - 1 // 0 indexed
        - 4; // ignore "\" + " \ " used by root dir

    let x = desired_x.min(width/2);

    // print_path(&path_buf, x, 0);
    print_ls(&path_buf, x + 10, height, None);

}

fn files_equal(a: &DirEntry, b: &DirEntry) -> bool {
    let metadata_a = a.metadata().unwrap();
    let metadata_b = b.metadata().unwrap();

    // if two files considered equal if they share a filename and date created
    a.file_name() == b.file_name() && metadata_a.created().unwrap() == metadata_b.created().unwrap()
}

fn get_y_above(path: &PathBuf, height: u16, last_seen: Option<DirEntry>) -> usize {
    let mut dirs = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());

    match last_seen {
        None => {
            for i in 0..height {
                if let None = dirs.next() {
                    return i as usize / 2;
                }
            }
        
            height as usize / 2
        }
        Some(last_seen) => {
            for (i, dir_entry) in dirs.enumerate() {
                if files_equal(&last_seen, &dir_entry) {
                    return i;
                }
            }

            get_y_above(path, height, None)
        }
    }
    
}

fn print_ls(path: &PathBuf, x: u16, height: u16, last_seen: Option<DirEntry>) -> Result<(), Error> {

    
    let above= get_y_above(path, height, last_seen);
    
    let y: i32 = height as i32/2 - above as i32;
    let mut dirs = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());

    for _ in y..0 { dirs.next(); } // offscreen

    let mut y = y.max(0) as u16;

    for dir in dirs {
        if y > height { return Ok(()); }
        print_at(dir.file_name().to_str().unwrap(), x, y)?;
        y += 1;
    }

    let files = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
    for file in files {
        if y > height { return Ok(()); }
        print_at(file.file_name().to_str().unwrap(), x, y)?;
        y += 1;
    }

    Ok(())
}

fn clear() {
    print!("{esc}c", esc = 27 as char);
}
fn print_at<T: Display>(str: T, x: u16, y: u16) -> Result<(), Error> {
    execute!(
        stdout(),
        cursor::MoveTo(x, y),
        Print(str),
    )
}

fn print_path(path: &PathBuf, mut x: u16, y: u16) -> Result<(), Error> {

    print_at(">", x, y)?;
    
    for dir in path.components().filter(|comp| Component::RootDir != *comp).rev() {
        let os_str = dir.as_os_str();
        
        if x as i32 - os_str.len() as i32 - 1 < 0 {
            return print_at("...", 0.max(x as i32 - 4) as u16, y);
        }
        x -= os_str.len() as u16 + 1;
        print_at(os_str.to_str().unwrap(), x, y)?;

        if let Component::Normal(_) = dir {
            if x as i32 - 2 < 0 {
                return print_at("...", 0, y);
            }
            x -= 2;
            print_at("\\", x, y)?;
        }
    }

    Ok(())

}