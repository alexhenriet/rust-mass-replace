use std::env;
use std::process;

use mass_replace::Config;

fn main() {
    let mut args = env::args();
    let program_name = args.next().unwrap();
    let error_message = || {
        format!(
            "SYNTAX => {} [-v] ORIG_STR RPLC_STR DIRECTORY_PATH",
            program_name
        )
    };
    let config = Config::new(args).unwrap_or_else(|_| {
        eprintln!("Error: {}", error_message());
        process::exit(1);
    });
    if let Err(err) = mass_replace::run(config) {
        eprintln!("Application error: {}", err);
        process::exit(1);
    }
}
