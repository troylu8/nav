
use std::collections::HashMap;
use std::fmt::{Debug, Display};
use std::fs::DirEntry;
use std::path::{Component, Path, PathBuf};
use std::fs;
use std::io::{stdout, Error};

use crossterm::*;
use crossterm::style::Print;

pub struct Map {
    width: u16,
    width_half: u16,
    center: u16,

    height: u16,
    height_half: u16,

    last_seen: HashMap<PathBuf, DirEntry>,

    path: PathBuf,
    ls: Vec<DirEntry>,
    curr_i: usize,
    all_iter: Box<dyn Iterator<Item = DirEntry>>,
}
impl Debug for Map {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Map").field("width", &self.width).field("width_half", &self.width_half).field("height", &self.height).field("height_half", &self.height_half).field("last_seen", &self.last_seen).field("path", &self.path).field("ls", &self.ls).field("curr_i", &self.curr_i).finish()
    }
}
impl Map {

    pub fn new(path: PathBuf) -> Result<Self, Error> {

        let (width, height) = terminal::size().unwrap();
        
        let mut res = Self {
            width,
            width_half: width/2,
            center: width/2,
            height,
            height_half: height/2,

            last_seen: HashMap::new(),

            path,
            ls: vec![],
            curr_i: 0,
            all_iter: Map::iter_dirs_first(&Path::new(".").into()) // dummy value
        };

        res.update_ls()?;
        
        Ok(res)
    }

    fn files_equal(a: &DirEntry, b: &DirEntry) -> bool {
        // if two files considered equal if they share a name and type
        a.file_name() == b.file_name() && a.file_type().unwrap() == b.file_type().unwrap()
    }

    fn iter_dirs_first(path: &PathBuf) -> Box<dyn Iterator<Item = DirEntry>> {
        let dirs = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());
        let files = fs::read_dir(path).unwrap().map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
        Box::new(dirs.chain(files))
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<(), Error> {
        self.path = path;
        self.update_ls()
    }

    fn update_ls(&mut self) -> Result<(), Error> {

        let desired_x = self.path.iter().fold(0, |acc, x| acc + x.len() + 3) as u16 // include " \ " after every component
            - 1 // " >" is behind the last component instead of " \ "
            - 1 // 0 indexed
            - 4; // ignore "\" + " \ " used by root dir
        self.center = desired_x.min(self.width_half);

        self.ls.clear();
        self.all_iter = Map::iter_dirs_first(&self.path);
        
        // populate ls
        match self.last_seen.get(&self.path) {
            None => {
                // add height amt of items into ls
                let mut dir_count = 0;
                for _ in 0..self.height {
                    if let Some(dir_entry) = self.all_iter.next() {
                        
                        if dir_entry.file_type()?.is_dir() {
                            dir_count += 1;
                        }
                        
                        self.ls.push(dir_entry);
                        
                    }
                    else { break };
                }
    
                self.curr_i = dir_count/2;
            }
            Some(last_seen) => {
                
                // read until last seen was found
                while let Some(dir_entry) = self.all_iter.next() {
                    if Map::files_equal(&last_seen, &dir_entry) {
                        self.curr_i = self.ls.len();
                        break;
                    }
                    else { break; }
                }

                // if last seen was found, push to ls..
                if let Some(dir_entry) = self.all_iter.next() {
                    self.ls.push(dir_entry);
                }
                // ..otherwise (the file was deleted), remove from last_seen and try again
                else { 
                    self.last_seen.remove(&self.path);
                    return self.update_ls();
                }

                self.fill_up();

                // add the remaining height/2 items to ls
                // for _ in 0..self.height/2 {
                //     if let Some(dir_entry) = self.all_iter.next() {
                //         self.ls.push(dir_entry);
                //     }
                //     else { break };
                // }

            }
        }

        Ok(())

    }

    fn print_ls(&self) -> Result<(), Error> {

        let start_i = 0.max(self.curr_i as i32 - self.height_half as i32) as usize;

        let start_y = 0.max(self.height_half as i32 - self.curr_i as i32) as usize;
        
        // min ( all the way to the bottom , amt of entries on screen + after )
        for d in 0..(self.height as usize).min(self.ls.len() - start_i) {
            print_at(self.ls[start_i + d].file_name().to_str().unwrap(), self.center + 4, (start_y + d) as u16)?;
        }

        Ok(())
    }

    /// adds files into `ls` to fill screen, if they exist
    fn fill_up(&mut self) {
        let needed = (self.curr_i as i32) + (self.height_half as i32) - (self.ls.len() as i32);
        for _ in 0..needed {
            if let Some(dir_entry) = self.all_iter.next() {
                self.ls.push(dir_entry);
            }
        }
    }

    fn print_path(&self) -> Result<(), Error> {

        let mut x = self.center;

        print_at(">", x, self.height_half)?;
        
        for dir in self.path.components().filter(|comp| Component::RootDir != *comp).rev() {
            let os_str = dir.as_os_str();
            
            if x as i32 - os_str.len() as i32 - 1 < 0 {
                return print_at("...", 0.max(x as i32 - 4) as u16, self.height_half);
            }
            x -= os_str.len() as u16 + 1;
            print_at(os_str.to_str().unwrap(), x, self.height_half)?;
    
            if let Component::Normal(_) = dir {
                if x as i32 - 2 < 0 {
                    return print_at("...", 0, self.height_half);
                }
                x -= 2;
                print_at("\\", x, self.height_half)?;
            }
        }
    
        Ok(())
    
    }


    pub fn print(&self) -> Result<(), Error> {
        clear();
        self.print_path()?;
        self.print_ls()?;

        Ok(())
    }

    pub fn move_into(&mut self) -> Result<(), Error> {
        if !self.ls.is_empty() {
            self.path.push(self.ls[self.curr_i].file_name());
            self.update_ls()?
        }

        Ok(())
    }

    pub fn move_out(&mut self) -> Result<(), Error> {
        if self.path.pop() {
            self.update_ls()?;
        }

        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.curr_i > 0 {
            self.curr_i -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if self.curr_i < self.ls.len()-1 {
            self.curr_i += 1;
            self.fill_up();
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

