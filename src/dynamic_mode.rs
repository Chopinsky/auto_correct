use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{AutoCorrect};
use candidate::Candidate;
use common::*;
use config::{AutoCorrectConfig, Config};
use crossbeam_channel as channel;
use threads_pool::*;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
}

pub(crate) fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    if let Err(e) = populate_words_set(&service.config, &service.pool) {
        eprintln!("Failed to initialize: {}", e);
        return;
    }
}

pub(crate) fn candidate(
    word: String,
    current_edit: u8,
    config: &Config,
    pool: Arc<ThreadPool>,
    mut tx_async: Option<channel::Sender<Candidate>>,
) -> Vec<Candidate> {

    let max_edit = config.get_max_edit();
    let defined_locale = config.get_locale();

    if current_edit >= max_edit {
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

            if let Some(ref tx) = tx_async {
                tx.send(candidate.clone());
            }

            results.push(candidate);
        }
    }

    // if a misspell, find the correct one within 1 edit distance
    let (tx, rx) = channel::unbounded();
    let to_sort = current_edit == 0;
    let current_edit = current_edit + 1;

    let (tx_next, tx_next_clone, rx_next) =
        if current_edit < max_edit {
            let (tx_raw, rx_raw) = channel::unbounded();
            let tx_raw_clone = tx_raw.clone();
            (Some(tx_raw), Some(tx_raw_clone), Some(rx_raw))
        } else {
            (None, None, None)
        };

    let word_clone = word.clone();
    let locale_clone = defined_locale.clone();
    let tx_clone = tx.clone();

    pool.execute(move || {
        if let Ok(set) = WORDS_SET.read() {
            delete_n_replace(
                word_clone,
                &set,
                locale_clone,
                current_edit,
                tx_clone,
                tx_next_clone,
            );
        }
    });

    pool.execute(move || {
        if let Ok(set) = WORDS_SET.read() {
            transpose_n_insert(
                word,
                &set,
                defined_locale,
                current_edit,
                tx,
                tx_next
            );
        }
    });

    let rx_next =
        if let Some(rx_chl) = rx_next {
            let (tx_raw, rx_raw) = channel::unbounded();
            let tx_async_clone = tx_async.clone();
            let config_moved = config.to_owned();

            pool.execute(move || {
                find_next_edit_candidates(current_edit, &config_moved, rx_chl, tx_raw, tx_async_clone);
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
        results.sort_by(|a, b| b.cmp(&a));
    }

    results
}

fn update_or_send(
    results: &mut Vec<Candidate>,
    candidate: Candidate,
    tx: &Option<channel::Sender<Candidate>>,
) -> bool {
    if !results.contains(&candidate) {
        results.push(candidate.clone());
        if let Some(tx_async) = tx {
            tx_async.send(candidate);
        }
    }

    true
}

fn find_next_edit_candidates(
    current_edit: u8,
    config: &Config,
    rx_chl: channel::Receiver<String>,
    tx: channel::Sender<Vec<Candidate>>,
    tx_async: Option<channel::Sender<Candidate>>,
) {
    let next_pool = Arc::new(ThreadPool::new(4));

    for next in rx_chl {
        let tx_async_clone = tx_async.clone();
        let candidates = candidate(
            next,
            current_edit,
            config,
            Arc::clone(&next_pool),
            tx_async_clone,
        );

        if candidates.len() > 0 {
            tx.send(candidates);
        }
    }
}

fn populate_words_set(config: &Config, pool: &ThreadPool) -> Result<(), String> {
    if let Ok(mut set) = WORDS_SET.write() {
        let (tx, rx) = channel::unbounded();
        let config_clone = config.clone();

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
