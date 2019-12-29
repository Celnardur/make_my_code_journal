use git2::{
    Repository,
    Commit,
    BranchType,
    Revwalk,
};

use std::{
    error::Error,
};

pub mod journal;
use journal::JournalDiff;
use journal::Config;
//use journal::Entry;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for repo in config.repos {
        let repo = Repository::open(repo)?;
        let walk = get_repo_revwalk(&repo)?;
        let commits = filter_by_email(&repo, walk, &config.emails)?;
        println!("{:?}", commits.len());
        for commit in commits {
            if let Ok(_journal_diff) = JournalDiff::from_commit(&repo, &commit) {
                //println!("{:?}\n", journal_diff);
            }
        }
    }

    Ok(())
}

fn get_repo_revwalk<'repo>(repo: &'repo Repository) -> Result<Revwalk<'repo>, Box<dyn Error>> {
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

fn filter_by_email<'repo>(repo: &'repo Repository, walk: Revwalk, emails: & Vec<String>) -> Result<Vec<Commit<'repo>>, Box<dyn Error>> {
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

