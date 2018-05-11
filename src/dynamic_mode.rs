
use std::collections::{HashMap, HashSet};
use std::sync::{mpsc, Once, RwLock, ONCE_INIT};

use super::{AutoCorrect, SupportedLocale};
use candidate::Candidate;
use common::*;
use threads_pool::*;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
}

static LAUNCH: Once = ONCE_INIT;

pub fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set(&service.pool, service.locale.clone()) {
            eprintln!("Failed to initialize: {}", e);
            return;
        }
    });
}

pub fn candidate(
    locale: SupportedLocale,
    word: String,
    current_edit: u8,
    max_edit: u8,
    pool: &ThreadPool,
) -> Vec<Candidate> {
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
            // TODO: keep searching even if word is a correct word
            let score = set[&word];
            return vec![Candidate::new(word, score, current_edit)];
        }
    }

    // if a misspell, find the correct one within 1 edit distance
    let (tx, rx) = mpsc::channel();
    let current_edit = current_edit + 1;

    let (tx_two, tx_two_clone, rx_two) = if current_edit < max_edit {
        let (tx_raw, rx_raw) = mpsc::channel();
        let tx_raw_clone = mpsc::Sender::clone(&tx_raw);
        (Some(tx_raw), Some(tx_raw_clone), Some(rx_raw))
    } else {
        (None, None, None)
    };

    let tx_clone = mpsc::Sender::clone(&tx);
    let word_clone = word.clone();
    let locale_clone = locale.clone();
    pool.execute(move || {
        delete_n_replace(locale_clone, word_clone, current_edit, tx_clone, tx_two_clone);
    });

    pool.execute(move || {
        transpose_n_insert(locale, word, current_edit, tx, tx_two);
    });

    //TODO: receive rx_two and initiate recursive call

    let mut result = Vec::new();
    for received in rx {
        println!("Count: {} vs. {}", current_edit, max_edit);

        if !result.contains(&received) {
            result.push(received);
        }
    }

    if result.len() > 1 {
        result.sort_by(|a, b| b.cmp(a));
    }

    result
}

fn populate_words_set(pool: &ThreadPool, locale: SupportedLocale) -> Result<(), String> {
    if let Ok(mut set) = WORDS_SET.write() {
        let (tx, rx) = mpsc::channel();

        pool.execute(move || {
            open_file_async(locale, tx);
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

fn delete_n_replace(
    locale: SupportedLocale,
    word: String,
    current_edit: u8,
    tx: mpsc::Sender<Candidate>,
    tx_two: Option<mpsc::Sender<HashSet<String>>>,
) {
    if let Ok(set) = WORDS_SET.read() {
        let edit_two = tx_two.is_some();

        let mut next_set: HashSet<String> = HashSet::new();
        let mut base: String;
        let mut replace: String;
        let mut removed: char;

        // deletes
        for pos in 0..word.len() {
            base = word.clone();
            removed = base.remove(pos);

            if edit_two && !base.is_empty() {
                next_set.insert(base.clone());
            }

            // replaces
            for chara in get_char_set(&locale) {
                if chara == removed {
                    continue;
                }

                replace = base.clone();
                replace.insert(pos, chara);

                if edit_two {
                    next_set.insert(replace.clone());
                }

                if set.contains_key(&replace) {
                    send_one_candidate(replace, current_edit, &set, &tx);
                }
            }

            if set.contains_key(&base) {
                send_one_candidate(base, current_edit, &set, &tx);
            }
        }

        if let Some(tx_edit_two) = tx_two {
            tx_edit_two
                .send(next_set)
                .expect("Failed to send the candidate to the caller");
        }
    }
}

fn transpose_n_insert(
    locale: SupportedLocale,
    word: String,
    current_edit: u8,
    tx: mpsc::Sender<Candidate>,
    tx_two: Option<mpsc::Sender<HashSet<String>>>,
) {
    if let Ok(set) = WORDS_SET.read() {
        let edit_two = tx_two.is_some();

        let mut next_set: HashSet<String> = HashSet::new();
        let mut base: String;
        let mut removed: char;

        // transposes
        for pos in 1..word.len() {
            base = word.clone();

            removed = base.remove(pos);
            base.insert(pos - 1, removed);

            if edit_two && !base.is_empty() {
                next_set.insert(base.clone());
            }

            if set.contains_key(&base) {
                send_one_candidate(base, current_edit, &set, &tx);
            }
        }

        // inserts
        for pos in 0..word.len() + 1 {
            for chara in get_char_set(&locale) {
                base = word.clone();
                base.insert(pos, chara);

                if edit_two {
                    next_set.insert(base.clone());
                }

                if set.contains_key(&base) {
                    send_one_candidate(base, current_edit, &set, &tx);
                }
            }
        }

        if let Some(tx_edit_two) = tx_two {
            tx_edit_two
                .send(next_set)
                .expect("Failed to send the candidate to the caller");
        }
    }
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
