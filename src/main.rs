
use std::borrow::Borrow;
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display};
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

    
    clear();


    let desired_x = path_buf.iter().fold(0, |acc, x| acc + x.len() + 3) as u16 // include " \ " after every component
        - 1 // " >" is behind the last component instead of " \ "
        - 1 // 0 indexed
        - 4; // ignore "\" + " \ " used by root dir

    let x = desired_x.min(width/2);

    // print_path(&path_buf, x, 0);
    let mut a= FileLibrary::new(&path_buf, height as usize);
    a.print(x + 10);

    // dbg!(a);

}



#[derive(Debug)]
struct FileLibrary {
    curr_ls: Listing,
    last_seen: HashMap<PathBuf, DirEntry>,
    height: usize,
    half_height: usize
}
impl FileLibrary {

    fn new(path: &PathBuf, height: usize) -> Self {
        Self {
            curr_ls: Listing::new(path, height, None),
            last_seen: HashMap::new(),
            height,
            half_height: height/2
        }
    }

    fn set_listing(&mut self, path: &PathBuf) {        
        self.curr_ls = Listing::new(path, self.height, self.last_seen.get(path));
    }

    fn files_equal(a: &DirEntry, b: &DirEntry) -> bool {
        // if two files considered equal if they share a name and type
        a.file_name() == b.file_name() && a.file_type().unwrap() == b.file_type().unwrap()
    }

    fn print(&mut self, x: u16) {
        
        // gap from bottom of ls to bottom of cmd
        let needed = (self.curr_ls.curr_i as i32) + (self.half_height as i32) - (self.curr_ls.ls.len() as i32);
        if needed > 0 {
            self.curr_ls.add(needed as usize);
        }

        let start_i = 0.max(self.curr_ls.curr_i as i32 - self.half_height as i32) as usize;

        let start_y = 0.max(self.half_height as i32 - self.curr_ls.curr_i as i32) as usize;
        
        // min ( all the way to the bottom , amt of entries on screen + after )
        for d in 0..(self.height).min(self.curr_ls.ls.len() - start_i) {
            print_at(self.curr_ls.ls[start_i + d].file_name().to_str().unwrap(), x, (start_y + d) as u16).unwrap();
        }
    }
}


struct Listing {
    ls: Vec<DirEntry>,
    curr_i: usize,
    all_iter: Box<dyn Iterator<Item = DirEntry>>,
}
impl Debug for Listing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Listing").field("ls", &self.ls).field("curr_i", &self.curr_i).finish()
    }
}
impl Listing {
    fn new(path: &PathBuf, height: usize, last_seen: Option<&DirEntry>) -> Self {
        
        let dirs = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());
        let files = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
        let all_iter = Box::new(dirs.chain(files));
        
        let mut res = Self {
            ls: vec![],
            curr_i: 0,
            all_iter
        };
        
        match last_seen {
            None => {
                // add height amt of items into ls
                let mut dir_count = 0;
                for _ in 0..height {
                    if let Some(dir_entry) = res.all_iter.next() {
                        
                        if dir_entry.file_type().unwrap().is_dir() {
                            dir_count += 1;
                        }
                        
                        res.ls.push(dir_entry);
                        
                    }
                    else { break };
                }

                res.curr_i = dir_count/2;
            }
            Some(last_seen) => {
                
                // read until last seen was found
                while let Some(dir_entry) = res.all_iter.next() {
                    if FileLibrary::files_equal(&last_seen, &dir_entry) {
                        res.curr_i = res.ls.len();
                        break;
                    }
                    else { res.ls.push(dir_entry); }
                }

                // if last seen was found, push to ls. otherwise, act as if it's None
                if let Some(dir_entry) = res.all_iter.next() {
                    res.ls.push(dir_entry);
                }
                else { return Listing::new(path, height, None); }

                // add the remaining height/2 items to ls
                for _ in 0..height/2 {
                    if let Some(dir_entry) = res.all_iter.next() {
                        res.ls.push(dir_entry);
                    }
                    else { break };
                }

            }
        }

        res
    }

    fn add(&mut self, amt: usize) {
        for _ in 0..amt {
            if let Some(dir_entry) = self.all_iter.next() {
                self.ls.push(dir_entry);
            }
        }
    }
}

// fn print_ls(path: &PathBuf, x: u16, height: u16, last_seen: Option<DirEntry>) -> Result<(), Error> {
    
//     let above= get_y_above(path, height, last_seen);
    
//     let y: i32 = height as i32/2 - above as i32;
//     let mut dirs = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());

//     for _ in y..0 { dirs.next(); } // offscreen

//     let mut y = y.max(0) as u16;

//     for dir in dirs {
//         if y > height { return Ok(()); }
//         print_at(dir.file_name().to_str().unwrap(), x, y)?;
//         y += 1;
//     }

//     let files = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
//     for file in files {
//         if y > height { return Ok(()); }
//         print_at(file.file_name().to_str().unwrap(), x, y)?;
//         y += 1;
//     }

//     Ok(())
// }

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