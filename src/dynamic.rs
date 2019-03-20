use crossbeam_channel as channel;
use hashbrown::{HashMap, HashSet};

use crate::AutoCorrect;
use crate::candidate::Candidate;
use crate::common;
use crate::config::Config;
use crate::config::SupportedLocale;

static mut DICT: Option<HashMap<String, u32>> = None;

pub(crate) fn initialize(service: &AutoCorrect) {
    if let Err(e) = populate_words_set(&service.config) {
        eprintln!("Failed to initialize: {}", e);
    }
}

pub(crate) fn candidate(
    word: String,
    edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    tx_async: &Option<channel::Sender<Candidate>>,
    marker: u32
) -> HashSet<Candidate> {
    if edit >= max_edit {
        return HashSet::new();
    }

    let word = word.trim().to_lowercase();
    if word.is_empty() {
        return HashSet::new();
    }

    // if already a correct word, we're done
    let mut results = if tx_async.is_none() {
        HashSet::with_capacity(2 * word.len())
    } else {
        HashSet::new()
    };

    if let Some(set) = dict_ref() {
        if set.contains_key(&word) {
            let candidate = Candidate::new(word.to_owned(), set[&word], edit);

            if let Some(tx) = tx_async {
                if let Err(_) = tx.send(candidate) {
                    return HashSet::new();
                }
            } else {
                results.insert(candidate);
            }
        }
    }

    // if a misspell, find the correct one within 1 edit distance
    let (tx, rx) = channel::bounded(64);
    let current_edit = edit + 1;

    let (tx_next, tx_next_clone, rx_next) =
        if current_edit < max_edit {
            let (tx_raw, rx_raw) = channel::bounded(256);
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
                tx_next_clone,
                marker
            );

/*
            common::deprecated::delete_n_replace(
                word_clone,
                set,
                current_edit,
                tx_clone,
                tx_next_clone
            )
*/
        }
    });

    let mut rx_next = rx_next.and_then(|chan| {
        let (tx_raw, rx_raw) = channel::bounded(16);
        let tx_async_clone = tx_async.clone();

        AutoCorrect::run_job(move || {
            find_next_edit_candidates(
                current_edit, max_edit, locale, chan, tx_raw, &tx_async_clone
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
                tx_next,
                marker
            );

/*
            common::deprecated::transpose_n_insert(
                word,
                set,
                current_edit,
                tx,
                tx_next
            );
*/
        }
    });

    {
        // move rx into the scope so it can drop afterwards
        for candidate in rx {
            if let Some(chan) = tx_async {
                if chan.send(candidate).is_err() {
                    return results;
                }
            } else {
                results.insert(candidate);
            }
        }
    }

    if let Some(chan) = rx_next.take() {
        for received in chan {
            if received.is_empty() {
                continue;
            }

            if tx_async.is_none() {
                // if using async channel, results have already been sent
                results.reserve(received.len());
                results.extend(received);
            }
        }
    }

    results
}

fn find_next_edit_candidates(
    edit: u8,
    max_edit: u8,
    locale: SupportedLocale,
    rx_next: channel::Receiver<(String, u32)>,
    tx: channel::Sender<HashSet<Candidate>>,
    tx_async: &Option<channel::Sender<Candidate>>,
) {
    for (next, marker) in rx_next {
        let candidates = candidate(
            next,
            edit,
            max_edit,
            locale,
            tx_async,
            marker
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

        set.shrink_to_fit();

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
            DICT.replace(
                HashMap::with_capacity(50_000)
            );
        }

        DICT.as_mut()
    }
}
