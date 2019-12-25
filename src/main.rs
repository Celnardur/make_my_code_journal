use std::process;
mod journal;
use journal::config::Config;

fn main() {
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            println!("Config error: {}", e);
            process::exit(1);
        },
    };

    if let Err(e) = git_journal::run() {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

