use std::error::Error;
use walkdir::WalkDir;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader, LineWriter};

pub struct Config {
  pub original: String,
  pub replacement: String,
  pub path: String,
  pub verbose: bool,
}

impl Config {
  pub fn new(args: &[String]) -> Result<Config,String> {
    let args_count: usize = args.len();
    if !(4..=5).contains(&args_count) {
      let message = format!("Syntax is {} [-v] orig_str rplc_str path", args[0]);
      return Err(message);
    }
    let (original, replacement, path, verbose) = parse_args(args);
    Ok(Config { original, replacement, path, verbose })
  }
}

pub fn parse_args(args: &[String]) -> (String, String, String, bool) {
  if args[1].starts_with("-v") {
    (args[2].clone(), args[3].clone(), args[4].clone(), true)
  } else {
    (args[1].clone(), args[2].clone(), args[3].clone(), false)
  }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
  if config.verbose {
    println!("Replacing '{}' by '{}' in folder '{}'", config.original, config.replacement, config.path);
  }
  for entry in WalkDir::new(config.path) {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }
    let mut path_str: String = path.to_string_lossy().to_string();
    if is_symlink(&path)? {
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
    let number_bytes = file_buffer.read_line(&mut buf)?;
    if number_bytes == 0 { // EOF
      break;
    } 
    if buf.contains(original) {
      return Ok(true);
    }
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
    let number_bytes = file_buffer.read_line(&mut buf)?;
    if number_bytes == 0 { // EOF
      break;
    } 
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

pub fn is_symlink(path: &std::path::Path) -> Result<bool,io::Error> {
  let metadata = fs::symlink_metadata(&path)?;
  let file_type = metadata.file_type();
  Ok(file_type.is_symlink())
}