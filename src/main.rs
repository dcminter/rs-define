use clap::Clap;
use std::ffi::OsString;
use std::fs::File;
use std::io::{BufReader, BufWriter, Error, ErrorKind};
use std::path::PathBuf;
use std::{env, io};

/// A simple tool for curation and lookup of definitions and for other dictionary-like purposes
#[derive(Clap)]
#[clap(version = env!("CARGO_PKG_VERSION"), author = env!("CARGO_PKG_AUTHORS"))]
struct Opts {
    /// The key
    key: String,

    /// The value to store in the dictionary
    definition: Option<String>,
}

static PREFERRED_PATHS: [&str; 3] = ["~/.define", "~/.config/define", "/etc/define"];

static DEFINITIONS_PATH_KEY: &str = "DEFINITIONS_DICTIONARY_PATH";

fn main() {
    std::process::exit(define());
}

fn define() -> i32 {
    env_logger::init();

    let options: Opts = Opts::parse();

    let result = match options.definition {
        None => lookup(options.key),
        Some(value) => store(options.key, value),
    };

    match result {
        Ok(_) => {
            log::debug!("Completed OK");
            0
        }
        Err(error) => {
            log::error!("Failed: {}", error);
            1
        }
    }
}

fn store(key: String, value: String) -> Result<(), Error> {
    log::debug!("Will store: {} with key {}", value, key);

    Ok(())
}

fn lookup(key: String) -> Result<(), Error> {
    log::debug!("Lookup: {}", &key);
    let candidate_paths = gather_candidate_paths(&key);
    log::debug!("Candidate paths: {:?}", candidate_paths);
    display_from_appropriate_path(candidate_paths, &key)
}

fn gather_candidate_paths(key: &String) -> Vec<PathBuf> {
    // Filter out any paths that
    //   * Don't exist
    //   * Aren't files
    let paths = match &env::var_os(DEFINITIONS_PATH_KEY) {
        Some(paths) => expand_supplied_paths(paths, &key),
        None => expand_default_paths(&key),
    };

    log::debug!("Will check paths: {:?}", paths);

    paths
        .into_iter()
        .filter(|p| p.exists())
        .filter(|p| p.is_file())
        .collect()
}

fn display_from_appropriate_path(candidate_paths: Vec<PathBuf>, key: &String) -> Result<(), Error> {
    // Look for the first candidate that can be read as a file and dump
    // that to the console
    for candidate_path in candidate_paths {
        match File::open(&candidate_path) {
            Err(error) => log::debug!("Error {:?} for path {:?}", error, &candidate_path),
            Ok(file) => {
                // println!("Success for path {:?}", &candidate_path);
                dump_file_to_console(file);
                return Ok(());
            }
        }
    }

    eprintln!("No definition found for '{}'", key);
    Err(Error::from(ErrorKind::NotFound))
}

fn dump_file_to_console(file: File) {
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(io::stdout());
    match io::copy(&mut reader, &mut writer) {
        Err(err) => eprintln!("ERROR: failed {:?}", err),
        Ok(value) => log::debug!("Ok, wrote {} bytes", value),
    }
}

fn expand_default_paths(key: &String) -> Vec<PathBuf> {
    PREFERRED_PATHS
        .to_vec()
        .into_iter()
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .map(|mut paths| {
            paths.push(key);
            paths
        })
        .collect()
}

fn expand_supplied_paths(paths: &OsString, key: &String) -> Vec<PathBuf> {
    env::split_paths(paths)
        .into_iter()
        .map(PathBuf::from)
        // Is there a more elegant way to implement the following closure?
        .map(|mut paths| {
            paths.push(key);
            paths
        })
        .collect()
}
