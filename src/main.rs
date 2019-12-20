use git2::{
    Repository,
    Oid, 
};

use std::error::Error;
use std::process;

fn main() {
   if let Err(e) = run() {
       println!("Application error: {}", e);
       process::exit(1);
   }
}

fn run() -> Result<(), Box<dyn Error>> {
    let path = "/home/celnardur/School/compilers/project";
    let repo = Repository::open(path)?;

    let old_oid = Oid::from_str("122ec37784464ae399e80d8c28dc5d4d0f04ca85")?;
    let new_oid = Oid::from_str("4073d50275f54d87b007054a2c02c8ff33b1541c")?;

    let old_commit = repo.find_commit(old_oid)?;
    let new_commit = repo.find_commit(new_oid)?;

    let old_tree = repo.find_tree(old_commit.tree_id())?;
    let new_tree = repo.find_tree(new_commit.tree_id())?;

    let diff = repo.diff_tree_to_tree(Some(&old_tree), Some(&new_tree), None)?;

    Ok(())
}
