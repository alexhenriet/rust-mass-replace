use std::env;
use std::error::Error;
use std::fs::{self, File};
use std::io::{self, prelude::*, BufReader, LineWriter, Seek, SeekFrom};
use walkdir::WalkDir;

pub struct Config {
    pub original: String,
    pub replacement: String,
    pub path: String,
    pub verbose: bool,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, ()> {
        let mut original = args.next().ok_or(())?;
        let mut verbose = false;
        if original.starts_with("-v") {
            verbose = true;
            original = args.next().ok_or(())?;
        }
        let replacement = args.next().ok_or(())?;
        let path = args.next().ok_or(())?;
        Ok(Config {
            original,
            replacement,
            path,
            verbose,
        })
    }
}

pub fn run(config: Config) -> Result<(), Box<dyn Error>> {
    if config.verbose {
        println!(
            "Replacing '{}' by '{}' in folder '{}'",
            config.original, config.replacement, config.path
        );
    }
    for entry in WalkDir::new(config.path) {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            continue;
        }
        let mut path_str = path.display().to_string();
        if fs::symlink_metadata(&path)?.file_type().is_symlink() {
            path_str = fs::read_link(&path)?.as_path().display().to_string();
        }
        let file = match File::open(&path_str) {
            Ok(f) => f,
            Err(_) => continue,
        };
        if let Ok(true) = file_contains_string(&file, &config.original) {
            let count = replace_in_file(&file, &path_str, &config.original, &config.replacement)?;
            if config.verbose {
                println!("{} line(s) replaced in {}", count, path_str);
            }
        }
    }
    Ok(())
}

pub fn file_contains_string(file: &File, original: &str) -> Result<bool, io::Error> {
    let mut file_buffer = BufReader::new(file);
    let mut buf = String::new();
    while file_buffer.read_line(&mut buf)? != 0 {
        if buf.contains(original) {
            return Ok(true);
        }
        buf.clear();
    }
    Ok(false)
}

pub fn replace_in_file(
    mut file: &File,
    path: &str,
    original: &str,
    replacement: &str,
) -> Result<i32, io::Error> {
    file.seek(SeekFrom::Start(0))?;
    let mut count = 0;
    let tmp_path = format!("{}.tmp", path);
    let tmp_file = File::create(&tmp_path)?;
    let mut tmp_file: LineWriter<File> = LineWriter::new(tmp_file);
    let permissions = fs::metadata(&path)?.permissions();
    let mut file_buffer = BufReader::new(file);
    let mut buf = String::new();
    while file_buffer.read_line(&mut buf)? != 0 {
        if buf.contains(original) {
            buf = buf.replace(&original, &replacement);
            count += 1;
        }
        tmp_file.write_all(buf.as_bytes())?;
        buf.clear();
    }
    tmp_file.flush()?;
    fs::remove_file(&path)?; // required on x86_64-pc-windows-gnu to prevent (os error 5)
    fs::rename(&tmp_path, &path)?;
    fs::set_permissions(&path, permissions)?;
    Ok(count)
}
