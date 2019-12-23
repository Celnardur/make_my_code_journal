use std::process;

fn main() {
   if let Err(e) = git_journal::run() {
       println!("Application error: {}", e);
       process::exit(1);
   }
}

