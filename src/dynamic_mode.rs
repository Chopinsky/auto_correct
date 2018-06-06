use std::collections::HashMap;
use std::sync::{mpsc, Arc, Once, RwLock, ONCE_INIT};

use super::{AutoCorrect, SupportedLocale};
use candidate::Candidate;
use common::*;
use config::{AutoCorrectConfig, Config};
use threads_pool::*;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
}

static LAUNCH: Once = ONCE_INIT;

pub fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    LAUNCH.call_once(|| {
        if let Err(e) = populate_words_set(&service.config, &service.pool) {
            eprintln!("Failed to initialize: {}", e);
            return;
        }
    });
}

pub fn candidate(
    word: String,
    current_edit: u8,
    config: &Config,
    pool: Arc<ThreadPool>,
    mut tx_async: Option<mpsc::Sender<Candidate>>,
) -> Vec<Candidate> {
    let defined_max_edit = config.get_max_edit();
    let defined_locale = config.get_locale();

    if current_edit >= defined_max_edit {
        return Vec::new();
    }

    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return Vec::new();
    }

    // if already a correct word, we're done
    let mut results = Vec::new();
    if let Ok(set) = WORDS_SET.read() {
        if set.contains_key(&word) {
            let score = set[&word];
            let candidate = Candidate::new(word.to_owned(), score, current_edit);

            if let Some(tx) = tx_async.take() {
                if let Err(_) = tx.send(candidate.clone()) {
                    // if the channel is closed, no need to continue the search, return here
                    return vec![candidate];
                }
            }

            results.push(candidate);
        }
    }

    // if a misspell, find the correct one within 1 edit distance
    let (tx, rx) = mpsc::channel();
    let to_sort = current_edit == 0;
    let current_edit = current_edit + 1;

    let (tx_next, tx_next_clone, rx_next) =
        if current_edit < defined_max_edit {
            let (tx_raw, rx_raw) = mpsc::channel();
            let tx_raw_clone = mpsc::Sender::clone(&tx_raw);
            (Some(tx_raw), Some(tx_raw_clone), Some(rx_raw))
        } else {
            (None, None, None)
        };

    let tx_clone = mpsc::Sender::clone(&tx);
    let word_clone = word.clone();
    let locale_clone = config.get_locale().clone();

    pool.execute(move || {
        delete_n_replace(
            locale_clone,
            word_clone,
            current_edit,
            tx_clone,
            tx_next_clone,
        );
    });

    pool.execute(move || {
        transpose_n_insert(
            defined_locale, 
            word, 
            current_edit, 
            tx, 
            tx_next
        );
    });

    let rx_next =
        if let Some(rx_chl) = rx_next {
            let (tx_raw, rx_raw) = mpsc::channel();
            let tx_async_clone = tx_async.clone();
            let config_moved = config.to_owned();

            pool.execute(move || {
                find_next_edit_candidates(
                    current_edit, 
                    config_moved, 
                    rx_chl, 
                    tx_raw, 
                    tx_async_clone
                );
            });

            Some(rx_raw)
        } else {
            None
        };

    for candidate in rx {
        if !update_or_send(&mut results, candidate, &tx_async) {
            // if caller has dropped the channel before getting all results, stop trying to send
            tx_async = None;
        }
    }

    if let Some(rx) = rx_next {
        for mut received in rx {
            let space = results.capacity() - results.len();
            if space < received.len() {
                results.reserve(received.len());
            }

            loop {
                if received.is_empty() {
                    break;
                }

                if let Some(candidate) = received.pop() {
                    if !update_or_send(&mut results, candidate, &tx_async) {
                        // if caller has dropped the channel before getting all results, stop trying to send
                        tx_async = None;
                    }
                }
            }
        }
    }

    if to_sort && results.len() > 1 {
        results.sort_by(|a, b| b.cmp(a));
    }

    results
}

fn update_or_send(
    results: &mut Vec<Candidate>,
    candidate: Candidate,
    tx: &Option<mpsc::Sender<Candidate>>) -> bool {

    if !results.contains(&candidate) {
        results.push(candidate.clone());
        if let Some(tx_async) = tx {
            if let Err(_) = tx_async.send(candidate) {
                // if error, means caller has closed the channel
                return false;
            }
        }
    }

    true
}

fn find_next_edit_candidates(
    current_edit: u8,
    config: Config,
    rx_chl: mpsc::Receiver<String>,
    tx: mpsc::Sender<Vec<Candidate>>,
    tx_async: Option<mpsc::Sender<Candidate>>
) {
    let mut candidates = Vec::new();
    let next_pool = Arc::new(ThreadPool::new(4));

    for next in rx_chl {
        let tx_async_clone = tx_async.clone();
        let mut new_candidates =
            candidate(next, current_edit, &config, Arc::clone(&next_pool), tx_async_clone);

        let space = candidates.capacity() - candidates.len();
        if space < new_candidates.len() {
            candidates.reserve(new_candidates.len());
        }

        loop {
            if new_candidates.is_empty() {
                break;
            }

            if let Some(next_candidate) = new_candidates.pop() {
                if !candidates.contains(&next_candidate) {
                    candidates.push(next_candidate);
                }
            }
        }
    }

    tx.send(candidates)
        .expect("Failed to send the next round of candidates");
}

fn populate_words_set(config: &Config, pool: &ThreadPool) -> Result<(), String> {
    if let Ok(mut set) = WORDS_SET.write() {
        let (tx, rx) = mpsc::channel();
        let config_clone = config.clone();

        //TODO: use user defined dict size
        pool.execute(move || {
            load_dict_async(config_clone, tx);
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
    tx_two: Option<mpsc::Sender<String>>,
) {
    if let Ok(set) = WORDS_SET.read() {
        let edit_two = tx_two.is_some();

        let mut base: String;
        let mut replace: String;
        let mut removed: char;

        // deletes
        for pos in 0..word.len() {
            base = word.clone();
            removed = base.remove(pos);

            if edit_two && !base.is_empty() {
                send_next_string(base.clone(), &tx_two);
            }

            // replaces
            for chara in get_char_set(&locale) {
                if chara == removed {
                    continue;
                }

                replace = base.clone();
                replace.insert(pos, chara);

                if edit_two {
                    send_next_string(replace.clone(), &tx_two);
                }

                if set.contains_key(&replace) {
                    send_one_candidate(replace, current_edit, &set, &tx);
                }
            }

            if set.contains_key(&base) {
                send_one_candidate(base, current_edit, &set, &tx);
            }
        }
    }
}

fn transpose_n_insert(
    locale: SupportedLocale,
    word: String,
    current_edit: u8,
    tx: mpsc::Sender<Candidate>,
    tx_two: Option<mpsc::Sender<String>>,
) {
    if let Ok(set) = WORDS_SET.read() {
        let edit_two = tx_two.is_some();

        let mut base: String;
        let mut removed: char;

        // transposes
        for pos in 1..word.len() {
            base = word.clone();

            removed = base.remove(pos);
            base.insert(pos - 1, removed);

            if edit_two && !base.is_empty() {
                send_next_string(base.clone(), &tx_two);
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
                    send_next_string(base.clone(), &tx_two);
                }

                if set.contains_key(&base) {
                    send_one_candidate(base, current_edit, &set, &tx);
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
        let service = super::AutoCorrect {
            config: Config::new(),
            pool: Arc::new(ThreadPool::new(2)),
        };

        let _service = initialize(&service);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 5464);
    }
}
