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
    process,
    str,
    collections::HashSet,
    //io::prelude::*,
    //fs::File,
};

fn main() {
   if let Err(e) = run() {
       println!("Application error: {}", e);
       process::exit(1);
   }
}

fn run() -> Result<(), Box<dyn Error>> {
    let path = "git_test";
    let repo = Repository::open(path)?;
    let email = String::from("celnardur@protonmail.com");

    let commits = filter_commits(&repo, &email)?;
    println!("{:?}", commits);

    let old_oid = Oid::from_str("53b960426e5126ff65e71a62b743c9d17fcf1fbf")?;
    let new_oid = Oid::from_str("85f423d4a90650d2cd27b1c0d49fbd2ba92ab9a1")?;

    let diff = get_diff(&repo, old_oid, new_oid)?;
    diff.print(DiffFormat::Patch, diff_print)?;

    Ok(())
}

fn filter_commits<'repo>(repo: &'repo Repository, filter_email: &str) 
    -> Result<Vec<Commit<'repo>>, Box<dyn Error>> {
    let mut visited = HashSet::new();
    let mut commits = Vec::new();
    let mut walk = repo.revwalk()?;

    for branch in repo.branches(Some(BranchType::Local))? {
        let oid = match branch?.0.get().resolve()?.target() {
            Some(id) => id,
            None => continue,
        };

        walk.push(oid)?;
    }

    for oid in walk {
        let oid = oid?;
        if visited.contains(&oid) {
            continue
        }

        let commit = repo.find_commit(oid)?;
        let email = match commit.author().email() {
            Some(e) => String::from(e), 
            None => continue,
        };

        if email == filter_email {
            commits.push(commit);
        }

        visited.insert(oid);
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

fn diff_print(delta: DiffDelta, hunk: Option<DiffHunk>, line: DiffLine) -> bool {
    let content = match str::from_utf8(line.content()) {
        Err(_) => return false,
        Ok(s) => s, 
    };

    match line.origin() {
        '+' | '-' | ' ' => print!("{} {}", line.origin(), content),
        'F' => print!("\n{}", content),
        'H' => print!("{}", content),
        _ => (),
    }
    true
}
