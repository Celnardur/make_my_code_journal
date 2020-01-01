use git2::{
    Repository,
    Commit,
    BranchType,
    Revwalk,
};

use std::{
    error,
};

pub mod diffs;
pub use diffs::JournalDiff;
pub mod config;
pub use config::Config;
pub mod entry;
pub use entry::Entry;


// General Functions

pub fn get_repo_revwalk<'repo>(repo: &'repo Repository) -> Result<Revwalk<'repo>, Box<dyn error::Error>> {
    let mut walk = repo.revwalk()?;

    for branch in repo.branches(Some(BranchType::Local))? {
        let oid = match branch?.0.get().resolve()?.target() {
            Some(id) => id,
            None => continue,
        };

        walk.push(oid)?;
    }
    Ok(walk)
}

pub fn filter_by_email<'repo>(repo: &'repo Repository, walk: Revwalk, emails: & Vec<String>) -> Result<Vec<Commit<'repo>>, Box<dyn error::Error>> {
    let mut commits = Vec::new();
    for oid in walk {
        let commit = repo.find_commit(oid?)?;
        let is_match = match commit.author().email() {
            Some(e) => emails.contains(&String::from(e)),
            None => continue,
        };

        if is_match {
            commits.push(commit);
        }
    }
    Ok(commits)
}

// Error Class 

use std::fmt;
use std::string::String;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Error {
        Error { message: String::from(message) }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Journal Error: {}", self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}


