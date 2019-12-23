use std::{
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

    diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let content = match str::from_utf8(line.content()) {
            Err(_) => return false,
            Ok(s) => s, 
        };

        let mut entry = DiffInfo {
            origin: line.origin(),
            content: content.to_string(),
            new_file: String::new(),
            old_file: String::new(),
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
    pub files: Vec<FileChanges>,
}


impl JournalDiff {
    pub fn new() -> JournalDiff {
        JournalDiff {
            counts: LineCounts::new(),
            files: Vec::new(),
        }
    }

    pub fn construct(&mut self, diff: Diff) -> Result<(), Box<dyn Error>> {
        let mut info = Vec::new();
        get_diff_info(&mut info, diff)?;
        let mut index = 0;

        while info.get(index).is_some() {
            let mut changes = FileChanges::new();
            changes.construct(&info, &mut index);
            self.counts.add(&changes.counts);
            self.files.push(changes);
        }
        Ok(())
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

impl FileChanges {
    pub fn new() -> FileChanges {
        FileChanges {
            counts: LineCounts::new(),
            header: String::new(),
            old_path: String::new(),
            new_path: String::new(),
            hunks: Vec::new(),
        }
    }

    fn construct(&mut self, info: &Vec<DiffInfo>, index: &mut usize) {
        if info[*index].origin != 'F' {
            return;
        }

        self.header.push_str(&info[*index].content);
        self.old_path.push_str(&info[*index].old_file);
        self.new_path.push_str(&info[*index].new_file);
        *index += 1;
        while info.get(*index).is_some() && info[*index].origin == 'H' {
            let hunk = Hunk::new(info, index);
            self.counts.add(&hunk.counts);
            self.hunks.push(hunk);
        }
    }
}

#[derive(Debug)]
pub struct Hunk {
    pub counts: LineCounts,
    pub header: String,
    pub content: String,
}

impl Hunk {
    fn new(info: &Vec<DiffInfo>, index: &mut usize) -> Hunk {
        let mut hunk = Hunk {
            counts: LineCounts::new(),
            header: String::from(&info[*index].content),
            content: String::new(),
        };

        *index += 1;
        let mut origin = info[*index].origin;
        loop {
            match origin {
                '+' => hunk.counts.added += 1,
                '-' => hunk.counts.deleted += 1,
                ' ' => (),
                _ => break,
            }
            hunk.content.push_str(&format!("{} {}", origin, info[*index].content));
            *index += 1;
            origin = match info.get(*index) {
                Some(i) => i.origin,
                None => break,
            }
        }
        hunk
    }
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

    fn add(&mut self, rhs: &LineCounts) {
        self.added += rhs.added;
        self.deleted += rhs.deleted;
        self.modified += rhs.modified;
    }
}

