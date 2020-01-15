use git2::Repository;
use mmcj::*;
use std::{io, io::Write, process};
use termion::{color, event::Key, input::TermRead, raw::IntoRawMode};

fn main() {
    /*
    let config = Config::default();
    config.save("default_config.json");
    */
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            println!("Config error: {}", e);
            process::exit(1);
        }
    };

    if let Err(e) = run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

// Application Logic

pub fn run(config: Config) -> Result<(), Box<dyn std::error::Error>> {
    let mut diffs: Vec<Box<dyn Expand>> = Vec::new();

    for repo in &config.repos {
        let repo = Repository::open(repo)?;
        let walk = get_repo_revwalk(&repo)?;
        let commits = filter_by_email(&repo, walk, &config.emails)?;
        for commit in commits {
            if let Ok(journal_diff) = JournalDiff::from_commit(&repo, &commit) {
                diffs.push(Box::new(journal_diff));
            }
        }
    }

    let mut fl = FoldingList::new(diffs)?;
    let stdin = io::stdin();
    let mut stdout = io::stdout().into_raw_mode()?;
    let cs = config.get_color_settings()?;
    write!(stdout,"{}", fl.render(&cs)?)?;
    stdout.flush()?;

    for c in stdin.keys() {
        match c? {
            Key::Char('j') => fl.scroll(1),
            Key::Char('k') => fl.scroll(-1),
            Key::Char('f') => fl.collapse(),
            Key::Char('d') => fl.expand(),
            Key::Char('q') => break,
            _ => continue,
        }
        write!(stdout,"{}", fl.render(&cs)?)?;
        stdout.flush()?;
    }
    Ok(())
}
