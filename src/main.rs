use std::env;
use std::process;

use mass_replace::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });
    if let Err(err) = mass_replace::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1);
    }
}
