use std::{env, fs, io};
use std::borrow::{Borrow, Cow};
use std::ffi::OsString;
use std::fs::{DirEntry, File, ReadDir};
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader, BufWriter, Error, ErrorKind, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::collections::HashSet;
use std::iter::FromIterator;

use ansi_term::Colour::{Green, Red};
use atty::Stream::{Stderr, Stdout};
use clap::Parser;
use clap::ArgGroup;
use env_logger::Env;
use regex::Regex;

/// A simple tool for curation and lookup of definitions and for other dictionary-like purposes
#[derive(Parser, Debug)]
#[command(version = env ! ("CARGO_PKG_VERSION"), author = env ! ("CARGO_PKG_AUTHORS"), about, long_about = None)]
#[clap(group(ArgGroup::new("define_mode").multiple(true).conflicts_with("list_mode")))]
#[clap(group(ArgGroup::new("list_mode").conflicts_with("define_mode")))]
struct Opts {
    /// The key
    #[arg(group = "define_mode")]
    key: Option<String>,

    /// The value to store in the dictionary
    #[arg(group = "define_mode")]
    definition: Option<String>,

    /// Logging level (if any)
    #[arg(short, long)]
    logs: Option<String>,

    /// Disable lower-casing of dictionary keys
    #[arg(short, long, requires = "define_mode")]
    caseful: bool,

    /// List all known keys
    #[arg(long, group = "list_mode")]
    all: bool,
}

static DEFAULT_LOGGING_ENV_VAR: &str = "DEFINE_LOG";
static DEFAULT_LOGGING_LEVEL: &str = "off";

static PREFERRED_PATHS: [&str; 3] = ["~/.config/define", "~/.define", "/etc/define"];

static DEFINITIONS_PATH_KEY: &str = "DEFINITIONS_DICTIONARY_PATH";

fn main() {
    std::process::exit(define());
}

fn define() -> i32 {
    let options: Opts = Opts::parse();

    let level = options.logs.unwrap_or(DEFAULT_LOGGING_LEVEL.to_owned());

    env_logger::Builder::from_env(Env::default().filter_or(DEFAULT_LOGGING_ENV_VAR, level)).init();

    // TODO: Let's do some validation here and then we can make the next bit cleaner:
    if options.all && options.key.is_some() {}

    // TODO: This got messy... figure out how to tidy it up!
    let output = if options.all {
        let _ = list_everything();
        0
    } else {
        let output = match options.key {
            None => {
                // TODO:
                0
            }
            Some(key) => {
                // TODO:
                log::debug!("Key was: {}", key);

                let cased_key = if options.caseful {
                    key
                } else {
                    key.to_lowercase()
                };

                let result = match options.definition {
                    None => lookup(cased_key.as_str()),
                    Some(value) => store(cased_key.as_str(), value.as_str()),
                };

                let output = match result {
                    Ok(_) => {
                        log::debug!("Completed OK");
                        0
                    }
                    Err(error) => {
                        log::error!("Failed: {}", error);
                        1
                    }
                };
                output
            }
        };
        output
    };
    output
}

fn list_everything() -> Result<(), Error> {
    let possible_content_paths: Vec<PathBuf> = list_content_paths().into_iter().filter(|path| path.is_dir()).collect();
    log::debug!("Content paths (existing): {:?}", possible_content_paths);

    // TODO: Error handling here is a bit sketchy
    let terms:HashSet<String> = possible_content_paths.into_iter()
        .map(|path :PathBuf| path.read_dir().unwrap())
        .map(|directory :ReadDir | directory.map(|entry| entry.unwrap()))
        .flatten()
        .filter(|entry| entry.file_type().unwrap().is_file())
        .map(|entry| entry.file_name())
        .map(|name| name.to_string_lossy().to_string())
        .collect();
    log::debug!("Refined down to unique term keys: {:?}", terms);

    // Sort them lexically
    let mut sorted_terms = Vec::from_iter(terms);
    sorted_terms.sort();
    log::debug!("Sorted terms: {:?}", sorted_terms);

    // TODO: At this point I need to refactor things so that I can lookup arbitrary keys! Currently
    // the lookup is expecting to get a path and do the rendering itself.

    Ok(())
}

fn store(key: &str, value: &str) -> Result<(), Error> {
    log::debug!("Will store: {} with key {}", value, key);

    let candidate_paths = gather_candidate_paths(&key);
    log::debug!("Candidate paths: {:?}", candidate_paths);

    store_on_appropriate_path(candidate_paths, value)
}

fn store_on_appropriate_path(candidate_paths: Vec<PathBuf>, value: &str) -> Result<(), Error> {
    for candidate_path in &candidate_paths {
        match materialize_path(&candidate_path) {
            Ok(_) => {
                log::debug!(
                    "Successfully created path {:?} or it already exists",
                    candidate_path
                )
            }
            Err(_) => {
                log::debug!("Couldn't create path {:?}", candidate_path);
                continue;
            }
        }

        match OpenOptions::new()
            .write(true)
            .create(true)
            .read(true)
            .append(true)
            .open(candidate_path)
        {
            Err(error) => {
                log::debug!("Error {:?} for path {:?}", error, &candidate_path);
            }
            Ok(mut file) => {
                log::debug!("Opened file for appending on path {:?}", &candidate_path);
                if file.contains_text(&file, &value) {
                    log::debug!("File already contains the value, dumping to console");
                    file.seek(SeekFrom::Start(0))?;
                    dump_file_to_output(file, &mut io::stdout())?;
                    return Ok(());
                } else {
                    log::debug!(
                        "File does not contain the value, adding and then dumping to console"
                    );
                    writeln!(&file, "{}", value)?;
                    file.flush()?;
                    file.seek(SeekFrom::Start(0))?;
                    dump_file_to_output(file, &mut io::stdout())?;
                    return Ok(());
                }
            }
        }
    }

    // Is this errorkind sensible?
    Err(Error::from(ErrorKind::NotFound))
}

fn materialize_path(path: &PathBuf) -> Result<(), Error> {
    log::debug!(
        "Create the path {} if that seems necessary",
        path.to_string_lossy()
    );

    if path.exists() {
        log::debug!("Path {} exists already", path.to_string_lossy());
        Ok(())
    } else {
        log::debug!(
            "Path {} does not exist, will try to create the parent if necessary",
            path.to_string_lossy()
        );
        match path.parent() {
            Some(parent) => {
                if parent.exists() {
                    log::debug!("Parent path {} exists already", parent.to_string_lossy());
                    Ok(())
                } else {
                    log::debug!(
                        "Parent path {} doesn't exist - trying to create it",
                        parent.to_string_lossy()
                    );
                    fs::create_dir_all(parent)
                }
            }
            None => Ok(()),
        }
    }
}

// This is probably overkill, I just wanted to try it :)
trait ContainsText {
    fn contains_text(&self, file: &File, text: &str) -> bool;
}

impl ContainsText for File {
    fn contains_text(&self, file: &File, text: &str) -> bool {
        let reader = BufReader::new(file);

        // TODO: Ignore case
        let pattern: &str = &["^", &regex::escape(text), "$"].concat();

        // Are the following unwraps cool or should I be handling this more explicitly?
        let re = Regex::new(&pattern).unwrap();
        reader
            .lines()
            .into_iter()
            .any(|line| re.is_match(&line.unwrap()))
    }
}

fn lookup(key: &str) -> Result<(), Error> {
    log::debug!("Lookup: {}", &key);
    let candidate_paths = gather_candidate_read_paths(&key);
    log::debug!("Candidate paths: {:?}", candidate_paths);
    display_from_appropriate_path(candidate_paths, &key)
}

fn list_content_paths() -> Vec<PathBuf> {
    match &env::var_os(DEFINITIONS_PATH_KEY) {
        Some(paths) => expand_content_paths(paths),
        None => expand_default_content_paths(),
    }
}

fn gather_candidate_paths(key: &str) -> Vec<PathBuf> {
    match &env::var_os(DEFINITIONS_PATH_KEY) {
        Some(paths) => expand_supplied_paths(paths, &key),
        None => expand_default_paths(&key),
    }
}

fn gather_candidate_read_paths(key: &str) -> Vec<PathBuf> {
    gather_candidate_paths(&key)
        .into_iter()
        .filter(|p| p.exists())
        .filter(|p| p.is_file())
        .collect()
}

fn display_from_appropriate_path(candidate_paths: Vec<PathBuf>, key: &str) -> Result<(), Error> {
    // Look for the first candidate that can be read as a file and dump
    // that to the console
    for candidate_path in candidate_paths {
        match File::open(&candidate_path) {
            Err(error) => log::debug!("Error {:?} for path {:?}", error, &candidate_path),
            Ok(file) => {
                log::debug!("Success for path {:?}", &candidate_path);
                dump_file_to_output(file, &mut io::stdout())?;
                return Ok(());
            }
        }
    }

    // Is there a smart way to colour the whole string but still allow pattern
    // substitution? Also is there a better way to turn it off if we're not
    // talking to a tty?
    eprintln!(
        "No definition found for '{}'",
        if atty::is(Stderr) {
            Red.paint(key).to_string()
        } else {
            key.to_string()
        }
    );
    Err(Error::from(ErrorKind::NotFound))
}

fn dump_file_to_output(file: File, output: &mut dyn Write) -> Result<(), Error> {
    let mut reader = BufReader::new(file);
    let mut writer = BufWriter::new(output);
    if atty::is(Stdout) {
        write!(&mut writer, "{}", Green.prefix())?;
    }
    match io::copy(&mut reader, &mut writer) {
        Err(err) => {
            eprintln!("ERROR: failed {:?}", err);
            return Err(err);
        }
        Ok(value) => log::debug!("Ok, wrote {} bytes", value),
    }
    if atty::is(Stdout) {
        write!(&mut writer, "{}", Green.suffix())?;
    }
    Ok(())
}

fn expand_default_paths(key: &str) -> Vec<PathBuf> {
    PREFERRED_PATHS
        .to_vec()
        .into_iter()
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        // Is there a more elegant way to implement the following closure?
        .map(|mut paths| {
            paths.push(key);
            paths
        })
        .collect()
}

fn expand_default_content_paths() -> Vec<PathBuf> {
    PREFERRED_PATHS
        .to_vec()
        .into_iter()
        .map(|p| shellexpand::tilde(p).to_string())
        .map(PathBuf::from)
        .collect()
}

fn expand_supplied_paths(paths: &OsString, key: &str) -> Vec<PathBuf> {
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

fn expand_content_paths(paths: &OsString) -> Vec<PathBuf> {
    env::split_paths(paths)
        .into_iter()
        .map(PathBuf::from)
        .collect()
}

// Some unit tests... (only happy-path for now)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_supplied_paths() {
        // Given inputs...
        let input_paths = OsString::from("/tmp:/tmp/other:/tmp/other/stuff");
        let input_key = "FOO".to_string();

        // When we expand them...
        let output = expand_supplied_paths(&input_paths, &input_key);

        // We expect to get...
        let expected = vec![
            PathBuf::from("/tmp/FOO"),
            PathBuf::from("/tmp/other/FOO"),
            PathBuf::from("/tmp/other/stuff/FOO"),
        ];

        assert_eq!(output, expected);
    }

    #[test]
    fn test_expand_default_paths() {
        let output = expand_default_paths(&"FOO".to_string());

        let home = env::var_os("HOME").unwrap();

        let mut expected = vec![
            PathBuf::from(&home),
            PathBuf::from(&home),
            PathBuf::from("/etc/define/FOO"),
        ];

        expected[0].push(".config/define/FOO");
        expected[1].push(".define/FOO");

        assert_eq!(output, expected);
    }

    #[test]
    fn test_dump_file_to_output() {
        let content = File::open("tests/fixtures/example_content.txt").unwrap();
        let mut buffer = Vec::new();
        dump_file_to_output(content, &mut buffer).unwrap();

        let text = String::from_utf8_lossy(&buffer);

        let expected = "This is\nsome example\ncontent.";

        // Unfortunately the actual value will be affected by the console
        // type so the expectation must be too:
        if atty::is(Stdout) {
            assert_eq!(text, Green.paint(expected).to_string());
        } else {
            assert_eq!(text, expected)
        }
    }
}
