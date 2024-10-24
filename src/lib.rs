
use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{Debug, Display};
use std::fs::DirEntry;
use std::path::{Component, Path, PathBuf};
use std::time::Duration;
use std::{fs, thread};
use std::io::{stdout, Error};

use crossterm::*;
use crossterm::style::Print;

#[derive(Debug)]
struct FileDesc {
    name: OsString,
    is_dir: bool
}
impl PartialEq<DirEntry> for FileDesc {
    fn eq(&self, other: &DirEntry) -> bool {
        other.file_name() == self.name && other.file_type().unwrap().is_dir() == self.is_dir
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
    const CENTER_GAP: u16 = 4;

    pub fn new(path: PathBuf) -> Result<Self, Error> {

        let (width, height) = terminal::size().unwrap();
        
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
            all_iter: Map::iter_dirs_first(&Path::new(".").into())?
        };

        res.populate_ls()?;
        
        Ok(res)
    }

    fn iter_dirs_first(path: &PathBuf) -> Result<Box<dyn Iterator<Item = DirEntry>>, Error> {
        let dirs = fs::read_dir(path)?.map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());
        let files = fs::read_dir(path)?.map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
        Ok(Box::new(dirs.chain(files)))
    }

    pub fn set_path(&mut self, path: PathBuf) -> Result<(), Error> {
        match Map::iter_dirs_first(&path) {
            Ok(iter) => self.all_iter = iter,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::PermissionDenied {

                    let str = self.ls[self.curr_i].file_name().to_str().unwrap().to_string() + " [Access is denied.]";
                    set_focused_style(style::Color::DarkRed)?;
                    print_at(&str, self.center + Self::CENTER_GAP, self.height_half)?;
                }
                return Err(err);
            }
        };
        self.all_iter = Map::iter_dirs_first(&path)?;

        self.path = path;

        self.populate_ls()
    }

    /// assumes self.all_iter and self.path are correct
    fn populate_ls(&mut self) -> Result<(), Error> {

        self.ls.clear();
        
        let last_seen_key = &self.path.as_os_str().to_os_string();

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

                loop {
                    match self.all_iter.next() {
                        Some(dir_entry) => {

                            if last_seen == &dir_entry {
                                self.curr_i = self.ls.len();
                                self.ls.push(dir_entry);
                                self.fill_up();

                                return Ok(());
                            }

                            self.ls.push(dir_entry);
                        }

                        // made it to the end and last seen wasnt found, mustve been deleted
                        None => {
                            self.last_seen.remove(last_seen_key);
                            return self.populate_ls();
                        }
                    }
                }
                
            }
        }

        Ok(())

    }

    fn print_ls(&self) -> Result<(), Error> {

        let start_i = 0.max(self.curr_i as i32 - self.height_half as i32) as usize;

        let start_y = 0.max(self.height_half as i32 - self.curr_i as i32) as usize;
        let x = self.center + Self::CENTER_GAP;
        let entries_to_print = (self.height as usize).min(self.ls.len() - start_i);
        
        execute!(stdout(), style::SetAttribute(style::Attribute::Reset))?;

        // min ( all the way to the bottom , amt of entries on screen + after )
        for d in 0..entries_to_print {

            // non-directories should be dark grey
            if !self.ls[start_i + d].file_type()?.is_dir() {
                set_color(style::Color::DarkGrey)?;
            }

            
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

        // emphasize focused entry
        if !self.ls.is_empty() {
            
            if self.ls[self.curr_i].file_type()?.is_dir() {
                set_focused_style(style::Color::DarkYellow)?;
            }
            else {
                set_focused_style(style::Color::DarkGrey)?;
            }
            
            print_at(self.ls[self.curr_i].file_name().to_str().unwrap(), x, self.height_half)?;
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

        execute!(stdout(), style::SetAttribute(style::Attribute::Reset))?;
        set_color(style::Color::Cyan)?;

        clear_from(0, self.height_half, self.center as usize)?;


        print_at(">", x, self.height_half)?;
        
        for dir in self.path.components().filter(|comp| Component::RootDir != *comp).rev() {
            let os_str = dir.as_os_str();
            
            if x as i32 - os_str.len() as i32 - 1 < 0 {
                return print_at("...", 0.max(x as i32 - Self::CENTER_GAP as i32) as u16, self.height_half);
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
        
        if self.ls.is_empty() {
            set_focused_style(style::Color::DarkGrey)?;
            print_at("[Nothing here.]", self.center + Self::CENTER_GAP, self.height_half)?;
            
            return Ok(());
        }

        if !self.ls[self.curr_i].file_type()?.is_dir() {
            let str = self.ls[self.curr_i].file_name().to_str().unwrap().to_string() + " [Not a directory.]";
            set_focused_style(style::Color::DarkGrey)?;
            print_at(&str, self.center + Self::CENTER_GAP, self.height_half)?;

            return Ok(());
        }


        let file_name = self.ls[self.curr_i].file_name();

        self.set_path(self.path.join(&file_name))?;

        let new_center = 
            (self.center + file_name.len() as u16 + 3) // add " \ xxxxx"
            .min((self.width as f32 * 0.75) as u16);

        for y in 0..self.height {
            clear_from(self.center, y, (new_center - self.center + Self::CENTER_GAP) as usize)?;
        }

        self.center = new_center;

        self.print_path()?;
        self.print_ls()?;
        

        Ok(())
    }

    pub fn move_out(&mut self) -> Result<(), Error> {

        let focused_dir_entry = 
            if  self.ls.is_empty() { None }
            else { Some(&self.ls[self.curr_i]) };
        
        let name_before_pop = 
            if let Some(name) = self.path.file_name() { Some(name.to_os_string()) }
            else { None };

        let path_before_pop = self.path.as_os_str().to_os_string();

        if self.path.pop() {

            if let Some(focused_dir_entry) = focused_dir_entry {
                self.last_seen.insert(path_before_pop, FileDesc {
                    name: focused_dir_entry.file_name(),
                    is_dir: focused_dir_entry.file_type()?.is_dir()
                });
            }


            let path_after_pop = self.path.as_os_str().to_os_string();

            if let Some(name_before_pop) = name_before_pop {
                self.last_seen.insert(path_after_pop, FileDesc {
                    name: name_before_pop.to_os_string(),
                    is_dir: true
                });
            }

            self.all_iter = Map::iter_dirs_first(&self.path)?;
            self.populate_ls()?;

            self.center = Map::path_display_width(&self.path).min(self.width_half as usize) as u16;

            clear_row(">", self.center, self.height_half, self.center + Self::CENTER_GAP)?;
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

    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }
}


fn clear() -> Result<(), Error> {
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All)    
    )
}

fn set_color(color: style::Color) -> Result<(), Error> {
    execute!( stdout(), style::SetForegroundColor(color) )
}

fn set_focused_style(color: style::Color) -> Result<(), Error> {
    execute!( 
        stdout(), 
        style::SetAttribute(style::Attribute::Reset),
        style::SetAttribute(style::Attribute::Bold),
        style::SetForegroundColor(color),
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
        Print(" ".repeat(total_width as usize - str.len() - x as usize))
    )
}

