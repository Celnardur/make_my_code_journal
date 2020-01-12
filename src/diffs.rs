use crate::{folding_list::Expand, ColorSettings};
use git2::{Commit, Diff, Repository};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, error::Error, io::Stdout, io::Write, str};
use termion::{clear, color};

#[derive(Debug, Clone)]
struct DiffInfo {
    origin: char,
    content: String,
    file: String,
}

fn get_diff_info(info: &mut Vec<DiffInfo>, diff: Diff) -> Result<(), Box<dyn Error>> {
    let info_cell = RefCell::new(info);

    diff.print(git2::DiffFormat::Patch, |delta, _hunk, line| {
        let content = match str::from_utf8(line.content()) {
            Err(_) => return false,
            Ok(s) => s,
        };

        let mut entry = DiffInfo {
            origin: line.origin(),
            content: content.to_string(),
            file: String::new(),
        };
        if let Some(f) = delta.new_file().path_bytes() {
            entry.file = str::from_utf8(f).unwrap().to_string();
        }
        // git rid of trailing newline
        entry.content.pop();

        info_cell.borrow_mut().push(entry);
        true
    })?;
    Ok(())
}

// TODO: File rename appears as two seperate file changes, a deleted file and a made file
// TODO: Initial Commit needs some stuff for it
// TODO: Make way to make diff from Merge commit using seperate diffs for each merging branch
// TODO: Make Modified Lines register as modified instead of

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalDiff {
    counts: LineCounts,
    files: Vec<FileChanges>,
}

impl JournalDiff {
    pub fn new() -> JournalDiff {
        JournalDiff {
            counts: LineCounts::new(),
            files: Vec::new(),
        }
    }

    pub fn from_commit(repo: &Repository, commit: &Commit) -> Result<JournalDiff, Box<dyn Error>> {
        if commit.parent_count() > 1 {
            //TODO: Implement diff for a merge commit
            return Err(Box::new(super::Error::new(
                "Cannot git a diff for a merge commit yet",
            )));
        }

        let new_tree = repo.find_tree(commit.tree_id())?;

        let old_commit = repo.find_commit(commit.parent_id(0)?)?;
        let old_tree = repo.find_tree(old_commit.tree_id())?;
        JournalDiff::from_diff(repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?)
    }

    pub fn from_diff(diff: Diff) -> Result<JournalDiff, Box<dyn Error>> {
        let mut journal = JournalDiff::new();
        journal.construct(diff)?;
        Ok(journal)
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

impl Expand for JournalDiff {
    fn expand(&self) -> (Vec<Box<dyn Expand>>, bool) {
        let mut folds: Vec<Box<dyn Expand>> = Vec::new();
        for file in &self.files {
            folds.push(Box::new(file.clone()));
        }
        (folds, true)
    }

    fn display(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}",
            colors.bg_default,
            clear::CurrentLine,
        )?;
        self.counts.display(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }

    fn highlight(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}{}",
            colors.bg_highlight,
            clear::CurrentLine,
            colors.fg_highlight,
        )?;
        self.counts.highlight(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChanges {
    counts: LineCounts,
    header: String,
    path: String,
    hunks: Vec<Hunk>,
}

impl FileChanges {
    pub fn new() -> FileChanges {
        FileChanges {
            counts: LineCounts::new(),
            header: String::new(),
            path: String::new(),
            hunks: Vec::new(),
        }
    }

    fn construct(&mut self, info: &Vec<DiffInfo>, index: &mut usize) {
        if info[*index].origin != 'F' {
            return;
        }

        self.header.push_str(&info[*index].content);
        self.path.push_str(&info[*index].file);
        *index += 1;
        while info.get(*index).is_some() && info[*index].origin == 'H' {
            let hunk = Hunk::new(info, index);
            self.counts.add(&hunk.counts);
            self.hunks.push(hunk);
        }
    }
}

impl Expand for FileChanges {
    fn expand(&self) -> (Vec<Box<dyn Expand>>, bool) {
        let mut folds: Vec<Box<dyn Expand>> = Vec::new();
        for hunk in &self.hunks {
            folds.push(Box::new(hunk.clone()));
        }
        (folds, true)
    }

    fn display(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}{}{}",
            colors.bg_default,
            clear::CurrentLine,
            colors.fg_default,
            self.path,
        )?;
        self.counts.display(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }

    fn highlight(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}{}{}",
            colors.bg_highlight,
            clear::CurrentLine,
            colors.fg_highlight,
            self.path,
        )?;
        self.counts.highlight(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hunk {
    counts: LineCounts,
    header: String,
    lines: Vec<DiffLine>,
}

impl Hunk {
    fn new(info: &Vec<DiffInfo>, index: &mut usize) -> Hunk {
        let mut hunk = Hunk {
            counts: LineCounts::new(),
            header: String::new(),
            lines: Vec::new(),
        };
        
        // Remove line of context from header
        let mut header_line = info[*index].content.chars();
        let mut at_count = 0;
        while let Some(ch) = header_line.next() {
            hunk.header.push(ch);
            if ch == '@' {
                if at_count == 3 {
                    break;
                }
                at_count += 1;
            }
        }

        *index += 1;
        let mut origin = info[*index].origin;
        loop {
            match origin {
                '+' => {
                    hunk.counts.added += 1;
                    hunk.lines
                        .push(DiffLine::Added(info[*index].content.clone()));
                }
                '-' => {
                    hunk.counts.deleted += 1;
                    hunk.lines
                        .push(DiffLine::Deleted(info[*index].content.clone()));
                }
                ' ' => hunk
                    .lines
                    .push(DiffLine::Context(info[*index].content.clone())),
                _ => break,
            }
            *index += 1;
            origin = match info.get(*index) {
                Some(i) => i.origin,
                None => break,
            }
        }
        hunk
    }
}

impl Expand for Hunk {
    fn expand(&self) -> (Vec<Box<dyn Expand>>, bool) {
        let mut folds: Vec<Box<dyn Expand>> = Vec::new();
        for line in &self.lines {
            folds.push(Box::new(line.clone()));
        }
        (folds, false)
    }

    fn display(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}{}{}",
            colors.bg_default,
            clear::CurrentLine,
            colors.fg_default,
            self.header,
        )?;
        self.counts.display(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }

    fn highlight(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream,
            "{}{}{}{}",
            colors.bg_highlight,
            clear::CurrentLine,
            colors.fg_highlight,
            self.header,
        )?;
        self.counts.highlight(stream, colors, line, width)?;
        write!(stream, "{}{}", color::Bg(color::Reset), color::Fg(color::Reset))?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum DiffLine {
    Context(String),
    Added(String),
    Deleted(String),
    //TODO: Add Moddified
}

impl Expand for DiffLine {
    fn display(&self, stream: &mut Stdout, colors: &ColorSettings, _line: u16, _width: u16) -> Result<(), Box<dyn Error>> {
        match self {
            DiffLine::Context(s) => {
                write!(
                    stream,
                    "{}{}{}  {}{}{}",
                    colors.bg_default,
                    clear::CurrentLine,
                    colors.fg_default,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
            DiffLine::Added(s) => {
                write!(
                    stream,
                    "{}{}{}+ {}{}{}",
                    colors.bg_default,
                    clear::CurrentLine,
                    colors.add,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
            DiffLine::Deleted(s) => {
                write!(
                    stream,
                    "{}{}{}- {}{}{}",
                    colors.bg_default,
                    clear::CurrentLine,
                    colors.delete,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
        };
        Ok(())
    }

    fn highlight(&self, stream: &mut Stdout, colors: &ColorSettings, _line: u16, _width: u16) -> Result<(), Box<dyn Error>> {
        match self {
            DiffLine::Context(s) => {
                write!(
                    stream,
                    "{}{}{}  {}{}{}",
                    colors.bg_highlight,
                    clear::CurrentLine,
                    colors.fg_highlight,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
            DiffLine::Added(s) => {
                write!(
                    stream,
                    "{}{}{}+ {}{}{}",
                    colors.bg_highlight,
                    clear::CurrentLine,
                    colors.add,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
            DiffLine::Deleted(s) => {
                write!(
                    stream,
                    "{}{}{}- {}{}{}",
                    colors.bg_highlight,
                    clear::CurrentLine,
                    colors.delete,
                    s,
                    color::Fg(color::Reset),
                    color::Bg(color::Reset),
                )?;
            }
        };
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineCounts {
    added: usize,
    deleted: usize,
    modified: usize,
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

    fn char_width(&self) -> u16 {
        let string = format!("{} {} {}", self.added, self.deleted, self.modified);
        string.len() as u16
    }

    fn display(&self, stream: &mut Stdout, colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream, 
            "{}{}{} {}{} {}{}",
            termion::cursor::Goto(width + 1 - self.char_width(), line),
            colors.add,
            self.added,
            colors.delete,
            self.deleted,
            colors.modified,
            self.modified,
        )?;
        Ok(())
    }

    fn highlight(&self, stream: &mut Stdout, _colors: &ColorSettings, line: u16, width: u16) -> Result<(), Box<dyn Error>> {
        write!(
            stream, 
            "{}{} {} {}",
            termion::cursor::Goto(width + 1 - self.char_width(), line),
            self.added,
            self.deleted,
            self.modified,
        )?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::Oid;

    fn get_diff_from_commit<'repo>(
        repo: &'repo Repository,
        oid: &str,
    ) -> Result<Diff<'repo>, Box<dyn Error>> {
        let new_commit = repo.find_commit(Oid::from_str(oid)?)?;
        let new_tree = repo.find_tree(new_commit.tree_id())?;

        let old_commit = repo.find_commit(new_commit.parent_id(0)?)?;
        let old_tree = repo.find_tree(old_commit.tree_id())?;
        Ok(repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?)
    }

    #[test]
    fn get_diff_info_test() -> Result<(), Box<dyn Error>> {
        let repo = Repository::open("mmcj_test_repo")?;

        let diff = get_diff_from_commit(&repo, "aeb35fe55b5deabc36399617c9a7c9281226b67e")?;
        let mut info = Vec::new();
        get_diff_info(&mut info, diff)?;

        assert_eq!(5, info.len());
        assert_eq!("@@ -1 +1,3 @@", info[1].content);
        assert_eq!("This Line Will Be Modified", info[4].content);

        assert_eq!('F', info[0].origin);
        assert_eq!('H', info[1].origin);
        assert_eq!(' ', info[2].origin);
        assert_eq!('+', info[3].origin);

        for di in info {
            assert_eq!(di.file, "a.txt");
        }

        Ok(())
    }
}
