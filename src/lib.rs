use git2::{
    Repository,
    Oid, 
    Diff,
    DiffDelta,
    DiffHunk,
    DiffLine,
    DiffFormat,
    Commit,
    BranchType,
    Revwalk,
};

use std::{
    error::Error,
    str,
    collections::HashSet,
    //io::prelude::*,
    //fs::File,
};

pub mod journal;
use journal::diffs::*;
use journal::config::Config;

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    for repo in config.repos {
        let repo = Repository::open(repo)?;
        let walk = get_repo_revwalk(&repo)?;
        let commits = filter_by_email(&repo, walk, &config.emails)?;
        println!("{:?}", commits.len());
        for commit in commits {
            if let Ok(journal_diff) = JournalDiff::from_commit(&repo, &commit) {
                println!("{:?}\n", journal_diff);
            }
        }
    }

    /*
    let old_oid = Oid::from_str("85f423d4a90650d2cd27b1c0d49fbd2ba92ab9a1")?;
    let new_oid = Oid::from_str("9c6fae26ae28db468d5111a608d29a672317fcfc")?;

    let diff = get_diff(&repo, old_oid, new_oid)?;
    diff.print(DiffFormat::Patch, diff_print)?;

    let counts = LineCounts::new();
    println!();

    let mut journal_diff = JournalDiff::new();
    journal_diff.construct(diff)?;
    println!("{}", serde_json::to_string(&journal_diff).unwrap());
    */

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

fn get_diff(repo: &Repository, old: Oid, new: Oid) -> Result<Diff, Box<dyn Error>> {
    let old_commit = repo.find_commit(old)?;
    let new_commit = repo.find_commit(new)?;

    let old_tree = repo.find_tree(old_commit.tree_id())?;
    let new_tree = repo.find_tree(new_commit.tree_id())?;

    Ok(repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?)
}

fn diff_print(delta: DiffDelta, _hunk: Option<DiffHunk>, line: DiffLine) -> bool {
    let content = match str::from_utf8(line.content()) {
        Err(_) => return false,
        Ok(s) => s, 
    };

    match line.origin() {
        '+' | '-' | ' ' => print!("{} {}", line.origin(), content),
        'F' => print!("\n{}", content),
        'H' => print!("  {}", content),
        _ => print!("{} {}", line.origin(), content),
    }
    true
}

