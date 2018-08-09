#![allow(unreachable_patterns)]

use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::Chars;

use super::SupportedLocale;
use candidate::Candidate;
use crossbeam_channel as channel;
use threads_pool::*;

pub static DELIM: &'static str = ",";
pub static DEFAULT_LOCALE: &'static str = "en-us";
static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";

pub(crate) fn find_variations(word: String, locale: SupportedLocale, pool: &ThreadPool) -> HashSet<String> {
    let (tx, rx) = channel::unbounded();
    let mut result = HashSet::new();

    pool.execute(move || {
        let len = word.len() + 1;
        for pos in 0..len {
            let source = word.clone();
            variations_at_pos(source, pos, len, locale, &tx);
        }
    });

    for variation_set in rx {
        for variation in variation_set {
            result.insert(variation);
        }
    }

    result
}

pub(crate)  fn delete_n_replace(
    word: String,
    set: &Box<HashMap<String, u32>>,
    locale: SupportedLocale,
    current_edit: u8,
    tx_curr: channel::Sender<Candidate>,
    tx_next: Option<channel::Sender<String>>,
) {
    let edit_two = tx_next.is_some();

    let mut base: String;
    let mut replace: String;
    let mut removed: char;

    // deletes
    for pos in 0..word.len() {
        base = word.clone();
        removed = base.remove(pos);

        if edit_two && !base.is_empty() {
            send_next_string(base.clone(), &tx_next);
        }

        // replaces
        for rune in get_char_set(&locale) {
            if rune == removed {
                continue;
            }

            replace = base.clone();
            replace.insert(pos, rune);

            if edit_two {
                send_next_string(replace.clone(), &tx_next);
            }

            if set.contains_key(&replace) {
                send_one_candidate(replace, current_edit, set, &tx_curr);
            }
        }

        if set.contains_key(&base) {
            send_one_candidate(base, current_edit, set, &tx_curr);
        }
    }
}

pub(crate) fn transpose_n_insert(
    word: String,
    set: &Box<HashMap<String, u32>>,
    locale: SupportedLocale,
    current_edit: u8,
    tx_curr: channel::Sender<Candidate>,
    tx_next: Option<channel::Sender<String>>,
) {
    let edit_two = tx_next.is_some();

    let mut base: String;
    let mut removed: char;

    // transposes
    let len = word.len() + 1;
    for pos in 0..len {
        if pos > 0 && pos < len - 1 {
            base = word.clone();

            removed = base.remove(pos);
            base.insert(pos - 1, removed);

            if edit_two && !base.is_empty() {
                send_next_string(base.clone(), &tx_next);
            }

            if set.contains_key(&base) {
                send_one_candidate(base, current_edit, set, &tx_curr);
            }
        }

        // inserts
        for rune in get_char_set(&locale) {
            base = word.clone();
            base.insert(pos, rune);

            if edit_two {
                send_next_string(base.clone(), &tx_next);
            }

            if set.contains_key(&base) {
                send_one_candidate(base, current_edit, set, &tx_curr);
            }
        }
    }
}

pub(crate) fn send_one_candidate(
    word: String,
    edit: u8,
    set: &Box<HashMap<String, u32>>,
    tx: &channel::Sender<Candidate>,
) {
    let score = set[&word];
    tx.send(Candidate::new(word, score, edit));
}

pub(crate) fn send_next_string(word: String, tx: &Option<channel::Sender<String>>) {
    if let Some(tx_next) = tx {
        tx_next.send(word);
    }
}

pub(crate) fn load_dict_async(dict_path: String, tx: channel::Sender<String>) {
    if dict_path.is_empty() {
        eprintln!("No dictionary path is given");
        return;
    }

    let file_loc = PathBuf::from(dict_path);
    if !file_loc.exists() || !file_loc.is_file() {
        eprintln!("Given dictionary path is invalid: {:?}", file_loc);
        return;
    }

    let file = File::open(file_loc).expect("file not found");
    let reader = BufReader::new(file);

    for raw_line in reader.lines() {
        if let Ok(line) = raw_line {
            tx.send(line);
        }
    }
}

pub(crate) fn get_char_set(locale: &SupportedLocale) -> Chars<'static> {
    match locale {
        &SupportedLocale::EnUs => ALPHABET_EN.chars(),
        _ => ALPHABET_EN.chars(),
    }
}

fn variations_at_pos(
    source: String,
    pos: usize,
    len: usize,
    locale: SupportedLocale,
    tx: &channel::Sender<HashSet<String>>
) {
    if pos >= len {
        return;
    }

    let mut result = HashSet::new();
    let mut word = source;

    // inserts
    for rune in get_char_set(&locale) {
        let mut base = word.clone();
        base.insert(pos, rune);

        result.insert(base);
    }

    if pos < len - 1 {
        // deletes
        let removed = word.remove(pos);
        result.insert(word.clone());

        // replaces
        for rune in get_char_set(&locale) {
            if rune == removed {
                continue;
            }

            let mut replace = word.clone();
            replace.insert(pos, rune);

            result.insert(replace);
        }

        // transposes
        if pos > 0 {
            word.insert(pos - 1, removed);
            result.insert(word.clone());
        }
    }

    if result.len() > 0 {
        tx.send(result);
    }
}