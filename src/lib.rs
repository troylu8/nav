
use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::fmt::{Debug, Display};
use std::fs::DirEntry;
use std::path::{Component, Path, PathBuf};
use std::fs;
use std::io::{stdout, Error, Write};

use crossterm::*;
use crossterm::style::Print;

#[derive(Debug)]
struct FileDesc {
    name: OsString,
    is_dir: bool
}
impl PartialEq<DirEntry> for FileDesc {
    fn eq(&self, other: &DirEntry) -> bool {
        other.file_name() == self.name && other.metadata().unwrap().is_dir() == self.is_dir
    }
}

pub struct Map {
    width: u16,
    width_half: u16,
    center: u16,

    height: u16,
    height_half: u16,

    last_seen: HashMap<OsString, FileDesc>,

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

        execute!(
            stdout(),
            crossterm::cursor::Hide,
            terminal::DisableLineWrap,
        )?;
        
        let mut res = Self {
            width,
            width_half: width/2,
            center: Map::path_display_width(&path).min(width as usize / 2) as u16, // start at middle or left
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

        

        self.ls.clear();
        self.all_iter = Map::iter_dirs_first(&self.path);
        
        let last_seen_key = &self.path.as_os_str().to_os_string();

        match self.last_seen.get(last_seen_key) {
            Some(file_desc) => print_at(file_desc.name.to_str().unwrap().to_string() + "        ", 0, 0)?,
            None => print_at("none       ", 0, 0)?,
        };
        

        // populate ls
        match self.last_seen.get(last_seen_key) {
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
                    if last_seen == &dir_entry {
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
                    self.last_seen.remove(last_seen_key);
                    return self.update_ls();
                }

                self.fill_up();
            }
        }

        Ok(())

    }

    fn print_ls(&self) -> Result<(), Error> {

        let start_i = 0.max(self.curr_i as i32 - self.height_half as i32) as usize;

        let start_y = 0.max(self.height_half as i32 - self.curr_i as i32) as usize;
        let x = self.center + 4;
        let entries_to_print = (self.height as usize).min(self.ls.len() - start_i);
        
        // min ( all the way to the bottom , amt of entries on screen + after )
        for d in 0..entries_to_print {
            clear_row(self.ls[start_i + d].file_name().to_str().unwrap(), x, (start_y + d) as u16, self.width)?;
        }

        // clear rows above
        for y in 0..start_y {
            clear_row("", x, y as u16, self.width)?;
        }

        // clear rows below
        for y in (start_y as u16 + entries_to_print as u16)..self.height {
            clear_row("", x, y as u16, self.width)?;
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

        clear_from(0, self.height_half, self.center as usize)?;

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
        clear()?;
        self.print_path()?;
        self.print_ls()?;

        Ok(())
    }

    pub fn move_into(&mut self) -> Result<(), Error> {
        if !self.ls.is_empty() && self.ls[self.curr_i].file_type()?.is_dir() {

            let file_name = self.ls[self.curr_i].file_name();

            let new_center = 
                (self.center + file_name.len() as u16 + 3) // add " \ xxxxx"
                .min((self.width as f32 * 0.75) as u16);

            for y in 0..self.height {
                clear_from(self.center, y, (new_center - self.center + 4) as usize)?;
            }

            self.center = new_center;

            self.path.push(file_name);

            self.update_ls()?;

            self.print_path()?;
            self.print_ls()?;
        }

        Ok(())
    }

    pub fn move_out(&mut self) -> Result<(), Error> {

        let file_desc = FileDesc {
            name: self.ls[self.curr_i].file_name(),
            is_dir: self.ls[self.curr_i].file_type()?.is_dir()
        };
        let last_seen_key = self.path.as_os_str().to_os_string();

        if self.path.pop() {

            self.last_seen.insert(last_seen_key, file_desc);

            self.update_ls()?;


            self.center = Map::path_display_width(&self.path).min(self.width_half as usize) as u16;

            clear_row(">", self.center, self.height_half, self.center + 4)?;
            self.print_ls()?;
            self.print_path()?;
        }

        Ok(())
    }

    pub fn move_up(&mut self) -> Result<(), Error> {
        if self.curr_i > 0 {
            self.curr_i -= 1;

            return self.print_ls();
        }

        Ok(())
    }

    pub fn move_down(&mut self) -> Result<(), Error> {
        if !self.ls.is_empty() && self.curr_i < self.ls.len()-1 {
            self.curr_i += 1;
            self.fill_up();

            return self.print_ls();
        }

        Ok(())
    }

    fn path_display_width(path: &PathBuf) -> usize {
        path.iter().fold(0, |acc, x| acc + x.len() + 3) // include " \ " after every component
            - 1 // " >" is behind the last component instead of " \ "
            - 1 // 0 indexed
            - 4 // ignore "\" + " \ " used by root dir
    }
}


fn clear() -> Result<(), Error> {
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All)    
    )
}
fn print_at<T: Display>(str: T, x: u16, y: u16) -> Result<(), Error> {
    execute!(
        stdout(),
        cursor::MoveTo(x, y),
        Print(str),
    )
}
fn clear_from(x: u16, y: u16, amt: usize) -> Result<(), Error> {
    execute!(
        stdout(),
        cursor::MoveTo(x, y),
        Print(" ".repeat(amt))
    )
}
fn clear_row(str: &str, x: u16, y: u16, total_width: u16) -> Result<(), Error> {
    execute!(
        stdout(),
        cursor::MoveTo(x, y),
        Print(str),
        Print("_".repeat(total_width as usize - str.len() - x as usize))
    )
}

