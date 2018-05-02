#![allow(dead_code)]

#[macro_use]
extern crate lazy_static;
extern crate threads_pool;

mod candidate;

pub mod prelude {
    pub use AutoCorrect;
    pub use candidate::Candidate;
}

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf};
use std::sync::{Once, ONCE_INIT, RwLock, mpsc};

use threads_pool::*;
use candidate::Candidate;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
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
        let pool = ThreadPool::new(2);
        let service = AutoCorrect {
            pool,
        };

        initialize(&service);
        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        // if already a correct word, we're done
        if let Ok(set) = WORDS_SET.read() {
            if set.contains_key(&word) {
                let score = set[&word];
                return vec![Candidate::new(word, score)];
            }
        }

        //TODO: configure to allow search 2 edit distance?

        // if a misspell, find the correct one within 1 edit distance
        let (tx, rx) = mpsc::channel();

        let tx_clone = mpsc::Sender::clone(&tx);
        let word_clone = word.clone();
        self.pool.execute(move || {
            search_combo_one(word_clone, tx_clone);
        });

        self.pool.execute(move || {
            search_combo_two(word, tx);
        });

        let mut result = Vec::new();
        for received in rx {
            if !result.contains(&received) {
                result.push(received);
            }
        }

        if result.len() > 1 {
            result.sort_by(|a, b| b.cmp(a));
        }
        
        result
    }
}

fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set(&service.pool) {
            eprintln!("Failed to initialize: {}", e);
            return;
        }

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
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

fn search_combo_one(word: String, tx: mpsc::Sender<Candidate>) {
    let mut transformed: String;

    if let Ok(set) = WORDS_SET.read() {
        // deletes
        for pos in 0..word.len() {
            transformed = word.clone();
            transformed.remove(pos);

            if set.contains_key(&transformed) {
                send_one_candidate(transformed, &set, &tx);
            }
        }

        // replaces
    }
}

fn search_combo_two(word: String, tx: mpsc::Sender<Candidate>) {
    // transposes

    // inserts
}

fn send_one_candidate(word: String, set: &Box<HashMap<String, u32>>, tx: &mpsc::Sender<Candidate>) {
    let score = set[&word];
    tx.send(Candidate::new(word, score)).expect("Failed to send the candidate to the caller");
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
