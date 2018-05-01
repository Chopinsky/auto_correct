#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate threads_pool;

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf};
use std::sync::{Once, ONCE_INIT, RwLock, mpsc};
use threads_pool::*;

pub mod prelude {
    pub use AutoCorrect;
}

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
    static ref INIT: RwLock<bool> = RwLock::new(false);
}

static DICTIONARY_PATH: &'static str = "./resources/words2.txt";
static DELIM: &'static str = "^";
static LAUNCH: Once = ONCE_INIT;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses
//TODO: customizable score function
//TODO: sort_by(|a, b| b.cmp(a));  <-- reverse sort, aka large elements at the front row

pub struct AutoCorrect {
    pool: ThreadPool,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        let pool = ThreadPool::new(3);
        let service = AutoCorrect {
            pool,
        };

        initialize(&service);
        service
    }
}

fn initialize(service: &AutoCorrect) {
    // if already initialized
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set(&service.pool) {
            eprintln!("Failed to initialize: {}", e);
            return;
        }

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)

        if let Ok(mut init) = INIT.write() {
            *init = true;
        }
    });
}

fn populate_words_set(pool: &ThreadPool) -> Result<(), String> {
    if let Ok(mut set) = WORDS_SET.write() {
        let (tx, rx) = mpsc::channel();

        pool.execute(move || {
            open_file_async(DICTIONARY_PATH, tx);
        });

        for received in rx {
            let temp: Vec<&str> = received.splitn(2, DELIM).collect();
            if temp[0].is_empty() {
                continue;
            }

            if let Ok(score) = temp[1].parse::<u32>() {
                let key = temp[0].to_owned();

                // if a larger score exists, use the larger score
                if set.contains_key(&key) && set[&key] >= score {
                    continue;
                }

                set.insert(key, score);
            }
        }

        return Ok(());
    }

    Err(String::from("Unable to write to the words set..."))
}

fn open_file_async(path: &str, tx: mpsc::Sender<String>) {
    if path.is_empty() { return; }

    let file_loc = PathBuf::from(path);
    if !file_loc.is_file() { return; }

    let file = File::open(file_loc).expect("file not found");
    let reader = BufReader::new(file);

    for raw_line in reader.lines() {
        if let Ok(line) = raw_line {
            if let Err(err) = tx.send(line) {
                println!("Unable to read the line from the file: {}", err);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_test() {
        let _service = AutoCorrect::new();

        let init = *INIT.read().unwrap();
        assert!(init);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 5464);
    }
}
