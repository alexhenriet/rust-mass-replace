use std::error::Error;
use walkdir::WalkDir;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader, LineWriter};
use std::env;

pub struct Config {
  pub original: String,
  pub replacement: String,
  pub path: String,
  pub verbose: bool,
}

impl Config {
  pub fn new(mut args: env::Args) -> Result<Config,String> {
    let error_message = format!("SYNTAX => {} [-v] ORIG_STR RPLC_STR DIRECTORY_PATH", args.next().unwrap());
    let mut original = match args.next() { None => return Err(error_message), Some(v) => v };
    let mut verbose = false;
    if original.starts_with("-v") { 
      verbose = true;
      original = match args.next() { None => return Err(error_message), Some(v) => v }; 
    }
    let replacement = match args.next() { None => return Err(error_message), Some(v) => v };
    let path = match args.next() { None => return Err(error_message), Some(v) => v };
    Ok(Config { original, replacement, path, verbose })
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  if config.verbose {
    println!("Replacing '{}' by '{}' in folder '{}'", config.original, config.replacement, config.path);
  }
  for entry in WalkDir::new(config.path) {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() { continue; }
    let mut path_str = path.display().to_string();
    if fs::symlink_metadata(&path)?.file_type().is_symlink() {  
      path_str = fs::read_link(&path)?.as_path().display().to_string();
    }
    if let Ok(found) = file_contains_string(&path_str, &config.original) {
      if found {
        let count = replace_in_file(&path_str, &config.original, &config.replacement)?;
        if config.verbose {
          println!("{} line(s) replaced in {}", count, path_str);
        }
      }
    }
  }
  Ok(())
}

pub fn file_contains_string(path: &str, original: &str) -> Result<bool,io::Error>
{
  let file = File::open(&path)?;
  let mut file_buffer = BufReader::new(file);
  let mut buf = String::new();
  loop {
    if 0 == file_buffer.read_line(&mut buf)? { break; } // EOF 
    if buf.contains(original) { return Ok(true); }
    buf.clear();
  }
  Ok(false)
}

pub fn replace_in_file(path: &str, original: &str, replacement: &str) -> Result<i32,io::Error>
{
  let mut count = 0;
  let tmp_path = format!("{}.tmp", path);
  let tmp_file = File::create(&tmp_path)?;
  let mut tmp_file: LineWriter<File> = LineWriter::new(tmp_file);
  let file = File::open(&path)?;
  let permissions = fs::metadata(&path)?.permissions();
  let mut file_buffer = BufReader::new(file);
  let mut buf = String::new();
  loop {
    if 0 == file_buffer.read_line(&mut buf)? { break; } // EOF 
    if buf.contains(original) {
      buf = buf.replace(&original, &replacement);
      count += 1;
    }
    tmp_file.write_all(buf.as_bytes())?;
    buf.clear();
  }
  tmp_file.flush()?;
  fs::rename(&tmp_path, &path)?;
  fs::set_permissions(&path, permissions)?;
  Ok(count)
}