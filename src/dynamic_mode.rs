
use std::collections::{HashMap, HashSet};
use std::sync::{Once, ONCE_INIT, RwLock, mpsc};

use threads_pool::*;
use candidate::Candidate;
use common::*;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
}

static LAUNCH: Once = ONCE_INIT;

pub fn initialize(pool: &ThreadPool) {
    // if already initialized, calling this function takes no effect
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set(pool) {
            eprintln!("Failed to initialize: {}", e);
            return;
        }
    });
}

pub fn candidate(word: String, current_edit: u8, max_edit: u8, pool: &ThreadPool) -> Vec<Candidate> {
    if current_edit >= max_edit {
        return Vec::new();
    }

    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return Vec::new();
    }

    // if already a correct word, we're done
    if let Ok(set) = WORDS_SET.read() {
        if set.contains_key(&word) {
            let score = set[&word];

            // TODO: keep searching even if word is a correct word
            return vec![Candidate::new(word, score, current_edit)];
        }
    }

    // if a misspell, find the correct one within 1 edit distance
    let (tx, rx) = mpsc::channel();
    let current_edit = current_edit + 1;

    let tx_clone = mpsc::Sender::clone(&tx);
    let word_clone = word.clone();
    pool.execute(move || {
        search_combo_one(word_clone, current_edit, tx_clone, None);
    });

    pool.execute(move || {
        search_combo_two(word, current_edit, tx, None);
    });

    let mut result = Vec::new();
    for received in rx {
        if !result.contains(&received) {
            result.push(received);
        }
    }

    //TODO: if current_edit < max_edit, recursive call candidate on every entry in the result vector

    if result.len() > 1 {
        result.sort_by(|a, b| b.cmp(a));
    }

    result
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

fn search_combo_one(word: String, current_edit: u8, tx: mpsc::Sender<Candidate>, tx_two: Option<mpsc::Sender<HashSet<String>>>) {
    let edit_two = tx_two.is_some();
    let mut begin_set: HashSet<String> = HashSet::new();
    let mut delete: String;

    if let Ok(set) = WORDS_SET.read() {
        // deletes
        for pos in 0..word.len() {
            delete = word.clone();
            delete.remove(pos);

            if edit_two && !delete.is_empty() {
                begin_set.insert(delete.clone());
            }

            // replaces
            let mut replace: String;
            for chara in ALPHABET.chars() {
                replace = delete.clone();
                replace.insert(pos, chara);

                if edit_two {
                    begin_set.insert(replace.clone());
                }

                if set.contains_key(&replace) {
                    send_one_candidate(replace, current_edit, &set, &tx);
                }
            }

            if set.contains_key(&delete) {
                send_one_candidate(delete, current_edit, &set, &tx);
            }
        }
    }

    if let Some(tx_edit_two) = tx_two {
        tx_edit_two.send(begin_set).expect("Failed to send the candidate to the caller");
    }
}

fn search_combo_two(word: String, current_edit: u8, tx: mpsc::Sender<Candidate>, tx_two: Option<mpsc::Sender<HashSet<String>>>) {
    // transposes

    // inserts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_test() {
        let pool = ThreadPool::new(2);
        let _service = initialize(&pool);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 5464);
    }
}
