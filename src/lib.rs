#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path};
use std::sync::{Once, ONCE_INIT, RwLock, mpsc};
use std::thread;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
    static ref INIT: RwLock<bool> = RwLock::new(false);
}

static DICTIONARY_PATH: &'static str = "../resources/words.txt";
static DELIM: &'static str = "^";
static LAUNCH: Once = ONCE_INIT;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses

fn initialize() {
    // if already initialized
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set() {
            eprintln!("Failed to initialize: {}", e);
            return;
        }

        if let Ok(mut init) = INIT.write() {
            *init = true;
        }
    });
}

fn populate_words_set() -> Result<(), String> {
    if let Ok(mut set) = WORDS_SET.write() {
        let (tx, rx) = mpsc::channel();

        //TODO: use pool!!!
        thread::spawn(move || {
            open_file_async(DICTIONARY_PATH, tx);
        });

        for received in rx {
            let temp: Vec<&str> = received.splitn(2, DELIM).collect();
            if temp[0].is_empty() {
                continue;
            }

            if let Ok(freq) = temp[1].parse::<u32>() {
                set.insert(temp[0].to_owned(), freq);
            }
        }

        return Ok(());
    }

    Err(String::from("Unable to write to the words set..."))
}

fn open_file_async(path: &str, tx: mpsc::Sender<String>) {
    if path.is_empty() {
        return;
    }

    let file_loc = Path::new(path);
    if !file_loc.is_file() {
        return;
    }

    if let Ok(file) = File::open(file_loc) {
        let mut reader = BufReader::new(file);
        for raw_line in reader.lines() {
            if let Ok(line) = raw_line {
                if let Err(err) = tx.send(line) {
                    println!("Unable to read the line from the file: {}", err);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_test() {
        initialize();

        let init = *INIT.read().unwrap();
        assert!(init);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 0); // TODO: use correct size
    }
}
