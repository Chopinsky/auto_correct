#![allow(unreachable_patterns)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::Chars;

use super::SupportedLocale;
use candidate::Candidate;
use config::Config;
use crossbeam_channel as channel;

pub static DELIM: &'static str = ",";
pub static DEFAULT_LOCALE: &'static str = "en-us";
static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";

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

pub(crate) fn load_dict_async(config: Config, tx: channel::Sender<String>) {
    let path = config.get_dict_path();

    if path.is_empty() {
        return;
    }

    let file_loc = PathBuf::from(path);
    if !file_loc.is_file() {
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
