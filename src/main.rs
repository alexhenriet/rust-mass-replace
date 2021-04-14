use std::env; 
use std::process;

use mass_replace::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });
    if let Err(err) = mass_replace::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1);
    }
}
