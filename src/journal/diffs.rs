use std::{
    ops, 
    collections::HashMap, 
    error::Error, 
    str,
    cell::RefCell,
};

use git2::{
    Diff,
};

#[derive(Debug, Clone)]
pub struct DiffInfo {
    origin: char,
    content: String,
    old_file: String,
    new_file: String,
}

pub fn get_diff_info(info: &mut Vec<DiffInfo>, diff: Diff) -> Result<(), Box<dyn Error>> {
    let info_cell = RefCell::new(info);

    diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
        let content = match str::from_utf8(line.content()) {
            Err(_) => return false,
            Ok(s) => s, 
        };

        let mut entry = DiffInfo {
            origin: line.origin(),
            content: content.to_string(),
            new_file: String::from(""),
            old_file: String::from(""),
        };

        match line.origin() {
            'F' => {
                if let Some(f) = delta.new_file().path_bytes() {
                    entry.new_file = str::from_utf8(f).unwrap().to_string();
                }
                if let Some(f) = delta.old_file().path_bytes() {
                    entry.old_file = str::from_utf8(f).unwrap().to_string();
                }
            },
            _ => (),
        }

        info_cell.borrow_mut().push(entry);
        true
    })?;
    Ok(())
}

#[derive(Debug)]
pub struct JournalDiff {
    pub counts: LineCounts,
    pub files: HashMap<String, FileChanges>,
}


impl JournalDiff {
    pub fn new(git_diff: Diff) -> Result<JournalDiff, Box<dyn Error>> {
        let mut diff = JournalDiff {
            counts: LineCounts::new(),
            files: HashMap::new(),
        };
        /*
        let mut file = String::from("None");
        let mut hunk_index = 0;

        // use refcell as a compiler promise to get around closure compiler limitations
        let diff_cell = RefCell::new(&diff);

        // couldn't find better interface to get all the data
        git_diff.print(git2::DiffFormat::Patch, |delta, hunk, line| {
            let content = match str::from_utf8(line.content()) {
                Err(_) => return false,
                Ok(s) => s, 
            };

            match line.origin() {
                '+' | '-' | ' ' => print!("{} {}", line.origin(), content),
                'F' => {
                    file = match delta.new_file().path_bytes() {
                        Some(p) => String::from(str::from_utf8(p).unwrap()),
                        None => String::from("None"),
                    };

                    let diff = diff_cell.borrow_mut();
                },
                'H' => print!("  {}", content),
                _ => print!("{} {}", line.origin(), content),
            }
            true
        })?;
        */
        Ok(diff)
    }
}

#[derive(Debug)]
pub struct FileChanges {
    pub counts: LineCounts,
    pub header: String,
    pub old_path: String,
    pub new_path: String,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug)]
pub struct Hunk {
    pub counts: LineCounts,
    pub header: String,
    pub content: String,
}

#[derive(Debug)]
pub struct LineCounts {
    pub added: usize,
    pub deleted: usize,
    pub modified: usize,
}

impl LineCounts {
    pub fn new() -> LineCounts {
        LineCounts {
            added: 0,
            deleted: 0,
            modified: 0,
        }
    }
}

impl ops::Add<LineCounts> for LineCounts {
    type Output = LineCounts;

    fn add(self, rhs: LineCounts) -> LineCounts {
        LineCounts {
            added: self.added + rhs.added,
            deleted: self.deleted + rhs.deleted,
            modified: self.modified + rhs.modified,
        }
    }
}
