/*
@TODO: 
- Use original EOL and not \n
- Use original file privileges to create new file
- Fix potentially unwanted last-line EOL (cat -vet Tool/Directory.php |less)
*/

use std::error::Error;
use walkdir::WalkDir;
use std::fs;
use std::fs::File;
use std::io::prelude::*;
use std::io::{self, BufRead, LineWriter};
use std::path::Path;

pub struct Config {
  pub original: String,
  pub replacement: String,
  pub path: String
}

impl Config {
  pub fn new(args: &[String]) -> Result<Config,String> {
      if args.len() < 4 {
          let message = format!("Syntax is {} $ORIGINAL $REPLACEMENT $PATH", args[0]);
          return Err(message);
      }
      let original = args[1].clone();
      let replacement = args[2].clone();
      let path = args[3].clone();
      Ok(Config { original, replacement, path })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  println!("REPLACING '{}' BY '{}' IN FOLDER '{}'", config.original, config.replacement, config.path);
  for entry in WalkDir::new(config.path) {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }
    let path_str = &path.to_string_lossy();
    if let Ok(found) = file_contains_string(path_str, &config.original) {
      if found {
        println!("Found in {}", path_str);
        let count = replace_in_file(path_str, &config.original, &config.replacement)?;
        println!("{} lines replaced", count);
      }
    }
  }
  Ok(())
}

pub fn file_contains_string(path: &str, original: &str) -> Result<bool,io::Error>
{
  if let Ok(lines) = read_lines(path) {
    for line in lines {
      let line = line?;
      if line.contains(original) {
        return Ok(true);
      }
    }
  }
  return Ok(false);
}

pub fn replace_in_file(path: &str, original: &str, replacement: &str) -> Result<i32,io::Error>
{
  let mut count = 0;
  let tmp_path = format!("{}.tmp", path);
  let tmp_file = File::create(&tmp_path)?;
  let mut tmp_file: LineWriter<File> = LineWriter::new(tmp_file);
  if let Ok(lines) = read_lines(path) {
    for line in lines {
      let line = line?;
      let mut to_write = format!("\n{}", line);
      if line.contains(original) {
        to_write = to_write.replace(&original, &replacement);
        count += 1;
      }
      tmp_file.write_all(to_write.as_bytes());
    }
    tmp_file.flush();
  }
  fs::rename(&tmp_path, &path)?;
  return Ok(count);
}

pub fn read_lines<P: AsRef<Path>>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}