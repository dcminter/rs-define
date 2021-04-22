use clap::Clap;
use std::{env, fs, io};
use std::ffi::OsString;
use std::path::PathBuf;
use std::fs::File;
use std::io::{BufReader, BufWriter};

/// A simple tool for curation and lookup of definitions and for other dictionary-like purposes
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    /// The key
    key: String,

    /// The value to store in the dictionary
    definition: Option<String>,
}

#[derive(PartialEq, Clone, Debug)]
struct Definition {
    key: String,
    definition: String,
}

static PREFERRED_PATHS: [&str; 3] = ["~/.define", "~/.config/define", "/etc/define"];

static DEFINITIONS_PATH_KEY: &str = "DEFINITIONS_DICTIONARY_PATH";

fn main() {
    let options: Opts = Opts::parse();
    match options.definition {
        None => lookup(options.key),
        Some(value) => store(options.key, value),
    }
}

fn store(key: String, value: String) {
    println!("Will store: {} with key {}", value, key);
}

fn lookup(key: String) {
    println!("Lookup: {}", &key);
    let paths = match &env::var_os(DEFINITIONS_PATH_KEY) {
        Some(paths) => expand_supplied_paths(paths, &key),
        None => expand_default_paths(&key),
    };

    println!("Will check paths: {:?}", paths);

    // Filter out any paths that
    //   * Don't exist
    //   * Aren't files

    let candidates: Vec<PathBuf> = paths.into_iter()
        .filter(|p| p.exists())
        .filter(|p| p.is_file())
        .collect();

    // Look for the first candidate that can be read as a file and dump
    // that to the console
    for candidate in candidates {
        match File::open(&candidate) {
            Err(_) => println!("Error for path {:?}", &candidate),
            Ok(file) => {
                println!("Success for path {:?}", &candidate);
                dump_file_to_console(file);
                break;
            }
        }
    }
}

fn dump_file_to_console(file:File) {
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(io::stdout());
    match io::copy(&mut reader, &mut writer) {
        Err(err) => eprintln!("ERROR: failed {:?}", err),
        Ok(value) => println!("Ok, wrote {} bytes", value)
    }
}

fn expand_default_paths(key: &String) -> Vec<PathBuf> {
    PREFERRED_PATHS
        .to_vec()
        .into_iter()
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|p| append_key(p, key))
        .collect()
}

fn expand_supplied_paths(paths: &OsString, key: &String) -> Vec<PathBuf> {
    env::split_paths(paths)
        .into_iter()
        .map(PathBuf::from)
        .map(|p| append_key(p, key))
        .collect()
}

// Surely I can do this in the iterator chain without needing a function?
fn append_key(mut path:PathBuf, key:&String) -> PathBuf {
    &path.push(key);
    path
}
