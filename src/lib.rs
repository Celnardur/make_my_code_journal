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
};

use std::{
    error::Error,
    str,
    collections::HashSet,
    //io::prelude::*,
    //fs::File,
};

mod journal;
use journal::diffs::*;

pub fn run() -> Result<(), Box<dyn Error>> {
    let path = "git_test";
    let repo = Repository::open(path)?;
    let email = String::from("celnardur@protonmail.com");

    let commits = filter_commits(&repo, &email)?;
    println!("{:?}", commits);

    let old_oid = Oid::from_str("85f423d4a90650d2cd27b1c0d49fbd2ba92ab9a1")?;
    let new_oid = Oid::from_str("9c6fae26ae28db468d5111a608d29a672317fcfc")?;

    let diff = get_diff(&repo, old_oid, new_oid)?;
    diff.print(DiffFormat::Patch, diff_print)?;

    let counts = LineCounts::new();
    println!();

    let mut journal_diff = JournalDiff::new();
    journal_diff.construct(diff)?;
    println!("{}", serde_json::to_string(&journal_diff).unwrap());

    Ok(())
}

fn filter_commits<'repo>(repo: &'repo Repository, email: &str) 
    -> Result<Vec<Commit<'repo>>, Box<dyn Error>> {
    let mut visited = HashSet::new();
    let mut walk = repo.revwalk()?;

    for branch in repo.branches(Some(BranchType::Local))? {
        let oid = match branch?.0.get().resolve()?.target() {
            Some(id) => id,
            None => continue,
        };

        walk.push(oid)?;
    }

    Ok(walk
       .filter(|oid| {
            let oid = match oid {
                Ok(id) => id,
                Err(_) => return false, 
            };

            if visited.contains(oid) {
                return false;
            }
            visited.insert(*oid);
            true
       })
       .map(|oid| return repo.find_commit(oid.unwrap()).expect("Couldn't find commit from oid"))
       .filter(|commit| {
           match commit.author().email() {
               Some(e) => e == email,
               None => false,
           }
       })
       .collect()
   )
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

