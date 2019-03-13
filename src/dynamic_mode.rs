use crate::AutoCorrect;
use crate::candidate::Candidate;
use crate::common;
use crate::config::Config;

use crossbeam_channel as channel;
use hashbrown::HashMap;
use config::SupportedLocale;

static mut DICT: Option<HashMap<String, u32>> = None;

pub(crate) fn initialize(service: &AutoCorrect) {
    if let Err(e) = populate_words_set(&service.config) {
        eprintln!("Failed to initialize: {}", e);
    }
}

pub(crate) fn enumerate(tx: channel::Sender<(String, u32)>) {
    if let Some(set) = dict_ref() {
        for (key, value) in set.iter() {
            tx.send((key.to_owned(), *value)).expect("Failed to send a dictionary word...");
        }
    }
}

pub(crate) fn candidate(
    word: String,
    current_edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    tx_async: &mut Option<channel::Sender<Candidate>>,
) -> Vec<Candidate> {
    if current_edit >= max_edit {
        return Vec::new();
    }

    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return Vec::new();
    }

    // if already a correct word, we're done
    let mut results = Vec::new();
    if let Some(set) = dict_ref() {
        if set.contains_key(&word) {
            let candidate = Candidate::new(word.to_owned(), set[&word], current_edit);

            if let Some(tx) = tx_async.as_ref() {
                tx.send(candidate.clone()).expect("Failed to send the search result...");;
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
    let tx_clone = tx.clone();

    AutoCorrect::run_job(move || {
        if let Some(set) = dict_ref() {
            common::ins_repl(
                &word_clone,
                set,
                current_edit,
                tx_clone,
                tx_next_clone
            );
        }
    });

    let mut rx_next = rx_next.and_then(|chan| {
        let (tx_raw, rx_raw) = channel::unbounded();
        let mut tx_async_clone = tx_async.clone();

        AutoCorrect::run_job(move || {
            find_next_edit_candidates(
                current_edit, max_edit, locale, chan, tx_raw, &mut tx_async_clone
            );
        });

        Some(rx_raw)
    });

    AutoCorrect::run_job(move || {
        if let Some(set) = dict_ref() {
            common::del_tran(
                &word,
                set,
                current_edit,
                tx,
                tx_next
            );
        }
    });

    {
        // move rx into the scope so it can drop afterwards
        for candidate in rx {
            if update_or_send(&mut results, candidate, &tx_async) {
                // if caller has dropped the channel before getting all results, stop trying to send
                *tx_async = None;
            }
        }
    }

    if let Some(chan) = rx_next.take() {
        for received in chan {
            if received.is_empty() {
                continue;
            }

            results.reserve(received.len());

            for candidate in received {
                if update_or_send(&mut results, candidate, &tx_async) {
                    // if caller has dropped the channel before getting all results, stop trying to send
                    *tx_async = None;
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
    let mut closed = false;
    if !results.contains(&candidate) {
        if let Some(tx_async) = tx {
            if tx_async.send(candidate.clone()).is_err() {
                closed = true;
            }
        }

        results.push(candidate);
    }

    closed
}

fn find_next_edit_candidates(
    current_edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    rx_chl: channel::Receiver<String>,
    tx: channel::Sender<Vec<Candidate>>,
    tx_async: &mut Option<channel::Sender<Candidate>>,
) {
    for next in rx_chl {
        let candidates = candidate(
            next,
            current_edit,
            max_edit,
            locale,
            tx_async,
        );

        if !candidates.is_empty() {
            tx.send(candidates).expect("Failed to send the search result...");;
        }
    }
}

fn populate_words_set(config: &Config) -> Result<(), String> {
    let (tx, rx) = channel::unbounded();
    let dict_path = config.get_dict_path();

    AutoCorrect::run_job(move || {
        common::load_dict_async(dict_path, tx);
    });

    if let Some(set) = dict_mut() {
        for received in rx {
            let temp: Vec<&str> = received.splitn(2, common::DELIM).collect();
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

#[inline]
fn dict_ref() -> Option<&'static HashMap<String, u32>> {
    unsafe { DICT.as_ref() }
}

#[inline]
fn dict_mut() -> Option<&'static mut HashMap<String, u32>> {
    unsafe {
        if DICT.is_none() {
            DICT.replace(HashMap::with_capacity(50_000));
        }

        DICT.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_test() {
        let service = super::AutoCorrect {
            config: Config::new(),
            //pool: Arc::new(ThreadPool::new(2)),
        };

        let _service = initialize(&service);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 5464);
    }
}
