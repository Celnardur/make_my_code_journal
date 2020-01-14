use git2::{BranchType, Commit, Repository, Revwalk};

use std::error;

pub mod diffs;
pub use diffs::JournalDiff;

pub mod config;
pub use config::Colors;
pub use config::Config;

pub mod entry;
pub use entry::Entry;

pub mod folding_list;
pub use folding_list::Expand;
pub use folding_list::FoldingList;

// General Functions

pub fn get_repo_revwalk<'repo>(
    repo: &'repo Repository,
) -> Result<Revwalk<'repo>, Box<dyn error::Error>> {
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

pub fn filter_by_email<'repo>(
    repo: &'repo Repository,
    walk: Revwalk,
    emails: &Vec<String>,
) -> Result<Vec<Commit<'repo>>, Box<dyn error::Error>> {
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

// Tests for General Functions
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_repo_revwalk_test() -> Result<(), Box<dyn error::Error>> {
        let repo = Repository::open("mmcj_test_repo")?;
        let walk = get_repo_revwalk(&repo)?;
        assert_eq!(
            12,
            walk.count(),
            "Returned revwalk has wrong number of commits"
        );
        Ok(())
    }

    #[test]
    fn filter_by_email_test() -> Result<(), Box<dyn error::Error>> {
        let repo = Repository::open("mmcj_test_repo")?;

        let walk = get_repo_revwalk(&repo)?;
        let both = vec![
            String::from("celnardur@protonmail.com"),
            String::from("celnardur@pm.com"),
        ];
        let commits = filter_by_email(&repo, walk, &both)?;
        assert_eq!(
            12,
            commits.len(),
            "Filtering test repo by two emails should return all commits"
        );

        let walk = get_repo_revwalk(&repo)?;
        let one = vec![String::from("celnardur@pm.com")];
        let commits = filter_by_email(&repo, walk, &one)?;
        assert_eq!(
            1,
            commits.len(),
            "Filtering test repo by celnardur@pm.com should return one commit"
        );
        Ok(())
    }
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
        Error {
            message: String::from(message),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mmcj Error: {}", self.message)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        None
    }
}
