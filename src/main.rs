use std::process;
use git_journal::journal::config::Config;

fn main() {
    let config = match Config::new() {
        Ok(c) => c,
        Err(e) => {
            println!("Config error: {}", e);
            process::exit(1);
        },
    };

    if let Err(e) = git_journal::run(config) {
        println!("Application error: {}", e);
        process::exit(1);
    }
}

