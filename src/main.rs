use std::process;
use mmcj::*;
use git2::Repository;
use termion::{
    color,
};

fn main() {
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            println!("Config error: {}", e);
            process::exit(1);
        },
    };

    if let Err(e) = run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

// Application Logic 

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut diffs = Vec::new();

    for repo in config.repos {
        let repo = Repository::open(repo)?;
        let walk = get_repo_revwalk(&repo)?;
        let commits = filter_by_email(&repo, walk, &config.emails)?;
        for commit in commits {
            if let Ok(journal_diff) = JournalDiff::from_commit(&repo, &commit) {
                diffs.push(journal_diff);
            }
        }
    }

    // throwaway code

    let (width, height) = termion::terminal_size()?;
    println!("{}{} {}", color::Fg(color::White), width, height);
    Ok(())
}
