use std::collections::HashMap;
use std::ffi::OsString;
use std::fmt::{Debug, Display};
use std::fs::DirEntry;
use std::path::{Component, PathBuf};
use std::fs;
use std::io::{stdout, Error};

use crossterm::*;
use crossterm::style::Print;


/// distance between "C:\ some \ path >" and the list of entries
const CENTER_GAP: u16 = 4;

#[derive(Debug, Clone)]
struct Entry {
    name: String,
    is_dir: bool
}
impl Entry {
    fn from_dir_entry(dir_entry: &DirEntry) -> Self {
        Self {
            name: get_filename_as_string(dir_entry),
            is_dir: dir_entry.file_type().unwrap().is_dir()
        }
    }
}
impl PartialEq<DirEntry> for Entry {
    fn eq(&self, other: &DirEntry) -> bool {
        get_filename_as_string(other) == self.name && 
        other.file_type().unwrap().is_dir() == self.is_dir
    }
}

fn get_filename_as_string(dir_entry: &DirEntry) -> String {
    dir_entry.file_name().as_os_str().to_str().unwrap().to_string()
}

#[derive(Debug, Default)]
pub struct Navigator {

    width: u16,
    height: u16,
    center_x: u16,

    /// some path -> entry we were looking at before moving out of this path
    last_seen: HashMap<OsString, Entry>,

    path: PathBuf,

    /// list of all entries in the current path. 
    entries: Vec<Entry>,

    query: Option<String>,
    filtered_entries: Option<Vec<Entry>>,

    /// index of the focused entry in `self.ls`
    pos: usize,

}

impl Navigator {
    /// initialize a new map
    pub fn new(path: PathBuf) -> Result<Self, Error> {

        let mut display = Navigator::default();
        display.set_size(terminal::size().unwrap());
        display.set_path(path)?;

        execute!( stdout(), terminal::Clear(terminal::ClearType::All) )?;
        display.print_path()?;
        display.print_entries()?;
        
        Ok(display)
    }

    /// get the current path
    pub fn get_path(&self) -> &PathBuf {
        &self.path
    }

    pub fn set_size(&mut self, (width, height): (u16, u16)) {
        self.width = width;
        self.height = height;
    }

    /// set the current path to `path`
    pub fn set_path(&mut self, path: PathBuf) -> Result<(), Error> {

        match iter_dirs_first(&path) {
            Ok(iter) => self.entries = iter.map(|dir_entry| Entry::from_dir_entry(&dir_entry)).collect(),
            Err(err) => {

                // display "permission denied" text
                if err.kind() == std::io::ErrorKind::PermissionDenied {
                    let str = self.entries[self.pos].name.clone() + " [Access is denied.]";
                    apply_focused_entry_style(style::Color::DarkRed)?;
                    print_at(&str, self.center_x + CENTER_GAP, self.height / 2)?;
                }

                return Err(err);
            }
        };

        self.center_x = path_display_width(&path)
            .min((self.width as f32 * 0.75) as usize) as u16; // path takes up 3/4ths of the screen at most
        
        self.path = path;
        self.pos = 0;
        self.update_visible_entries();

        Ok(())
    }

    pub fn set_query(&mut self, query: Option<String>) {
        self.query = query;
        self.pos = 0;        
        self.update_visible_entries();
    }

    fn update_visible_entries(&mut self) {
        //TODO: fuzzy
        self.filtered_entries = match &self.query {
            None => None,
            Some(query) => {
                Some(
                    self.entries.iter().filter_map(|entry| {
                        if &entry.name == query { Some(entry.clone()) }
                        else { None }
                    }).collect()
                )
            },
        };
    }

    /// print the current path
    fn print_path(&self) -> Result<(), Error> {
        execute!(stdout(), style::SetAttribute(style::Attribute::Reset))?;

        let mut x = self.center_x;
        let y = self.height / 2;

        execute!( stdout(), style::SetForegroundColor(style::Color::Cyan) )?;

        clear_from(0, y, self.center_x as usize)?;
        print_at(">", x, y)?;
        
        for dir in self.path.components().filter(|comp| Component::RootDir != *comp).rev() {
            let os_str = dir.as_os_str();
            
            if x as i32 - os_str.len() as i32 - 1 < 0 {
                return print_at("...", 0.max(x as i32 - CENTER_GAP as i32) as u16, y);
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

    pub fn print_entries(&self) -> Result<(), Error> {
        execute!(stdout(), style::SetAttribute(style::Attribute::Reset))?;

        let mid_y = self.height / 2;
         

        let mut entry_i = 0.max(self.pos as i32 - mid_y as i32) as usize;

        let x = self.center_x + CENTER_GAP;
        let mut y = 0.max(mid_y as i32 - self.pos as i32) as u16;

        // clear rows above
        for y in 0..y {
            clear_row("", x, y as u16, self.width)?;
        }

        let entries_to_print = match &self.filtered_entries {
            Some(filtered_entries) => filtered_entries,
            None => &self.entries
        };

        loop {
            if y >= self.height || entry_i >= entries_to_print.len() { break } 
            
            // non-directories should be dark grey
            if !entries_to_print[entry_i].is_dir {
                execute!( stdout(), style::SetForegroundColor(style::Color::DarkGrey) )?;
            }

            clear_row(&entries_to_print[entry_i].name, x, y, self.width)?;

            entry_i += 1;
            y += 1;
        }

        // clear rows below
        for y in y..self.height {
            clear_row("", x, y as u16, self.width)?;
        }

        // emphasize focused entry
        if !self.entries.is_empty() {
            
            if self.entries[self.pos].is_dir {
                apply_focused_entry_style(style::Color::DarkYellow)?;
            }
            else {
                apply_focused_entry_style(style::Color::DarkGrey)?;
            }
            
            print_at(&self.entries[self.pos].name, x, mid_y)?;
        }

        Ok(())
    }

    /// append the focused entry onto current path
    pub fn move_into(&mut self) -> Result<(), Error> {

        let mid_y = self.height / 2;
        
        if self.entries.is_empty() {
            apply_focused_entry_style(style::Color::DarkGrey)?;
            print_at("[Nothing here.]", self.center_x + CENTER_GAP, mid_y)?;
            
            return Ok(());
        }

        let focused_entry = &self.entries[self.pos];

        if !focused_entry.is_dir {
            let str = focused_entry.name.clone() + " [Not a directory.]";
            apply_focused_entry_style(style::Color::DarkGrey)?;
            print_at(&str, self.center_x + CENTER_GAP, mid_y)?;

            return Ok(());
        }


        let file_name = &focused_entry.name;

        let old_center_x = self.center_x;

        self.set_path(self.path.join(&file_name))?;

        for y in 0..self.height {
            clear_from(old_center_x, y, (self.center_x - old_center_x + CENTER_GAP) as usize)?;
        }

        self.print_path()?;
        self.print_entries()?;

        Ok(())
    }

    /// pop one item from the current path
    pub fn move_out(&mut self) -> Result<(), Error> {

        let focused_entry = 
            if  self.entries.is_empty() { None }
            else { Some(&self.entries[self.pos]) };
        
        let name_before_pop = 
            if let Some(name) = self.path.file_name() { Some(name.to_os_string()) }
            else { None };

        let path_before_pop = self.path.as_os_str().to_os_string();

        if self.path.pop() {

            if let Some(focused_entry) = focused_entry {
                self.last_seen.insert(path_before_pop, focused_entry.clone());
            }


            let path_after_pop = self.path.as_os_str().to_os_string();

            if let Some(name_before_pop) = name_before_pop {
                self.last_seen.insert(path_after_pop, Entry {
                    name: name_before_pop.as_os_str().to_str().unwrap().to_string(),
                    is_dir: true
                });
            }

            self.set_path(self.path.clone())?;

            clear_row(">", self.center_x, self.height / 2, self.center_x + CENTER_GAP)?;
            self.print_path()?;
            self.print_entries()?;
        }

        Ok(())
    }

    /// set focused entry to be the one above the current focused entry
    pub fn move_up(&mut self) -> Result<(), Error> {
        if self.pos > 0 {
            self.pos -= 1;

            return self.print_entries();
        }

        Ok(())
    }

    /// set focused entry to be the one below the current focused entry
    pub fn move_down(&mut self) -> Result<(), Error> {
        if !self.entries.is_empty() && self.pos < self.entries.len()-1 {
            self.pos += 1;

            return self.print_entries();
        }

        Ok(())
    }
}

fn apply_focused_entry_style(color: style::Color) -> Result<(), Error> {
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

/// print `str` at `x,y`, then clear from `x` -> `total_width`
fn clear_row(str: &str, x: u16, y: u16, total_width: u16) -> Result<(), Error> {
    execute!(
        stdout(),
        cursor::MoveTo(x, y),
        Print(str),
        Print(" ".repeat(total_width as usize - str.len() - x as usize))
    )
}



/// get an iterator at `path`, with directories before files
fn iter_dirs_first(path: &PathBuf) -> Result<Box<dyn Iterator<Item = DirEntry>>, Error> {
    let dirs = fs::read_dir(path)?.map(|x| x.unwrap()).filter(|x| x.file_type().unwrap().is_dir());
    let files = fs::read_dir(path)?.map(|x| x.unwrap()).filter(|x| !x.file_type().unwrap().is_dir());
    Ok(Box::new(dirs.chain(files)))
}

fn path_display_width(path: &PathBuf) -> usize {
    path.iter().fold(0, |acc, x| acc + x.len() + 3) // include " \ " after every component
        - 1 // " >" is behind the last component instead of " \ "
        - 1 // crossterm's Print(x,y) command is 0 indexed
        - 4 // ignore root dir "\"
}