#![allow(unused_variables)]

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::Chars;
use std::sync::mpsc;

use super::SupportedLocale;
use candidate::Candidate;

pub static DELIM: &'static str = "^";
pub static DEFAULT_LOCALE: &'static str = "en-us";
static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";

pub fn send_one_candidate(
    word: String,
    edit: u8,
    set: &Box<HashMap<String, u32>>,
    tx: &mpsc::Sender<Candidate>,
) {
    let score = set[&word];
    tx.send(Candidate::new(word, score, edit))
        .expect("Failed to send the candidate to the caller");
}

pub fn send_next_string(word: String, tx: &Option<mpsc::Sender<String>>) {
    if let Some(tx_next) = tx {
        tx_next
            .send(word)
            .expect("Failed to send the candidate to the caller");
    }
}

pub fn open_file_async(locale: SupportedLocale, tx: mpsc::Sender<String>) {
    let path = get_dict_path(locale);

    if path.is_empty() {
        return;
    }

    let file_loc = PathBuf::from(path);
    if !file_loc.is_file() {
        return;
    }

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

pub fn get_dict_path(locale: SupportedLocale) -> String {
    let locale = match locale {
        SupportedLocale::EnUs => "en-us",
        _ => "en-us",
    };

    format!("./resources/{}/words2.txt", locale)
}

pub fn get_char_set(locale: &SupportedLocale) -> Chars<'static> {
    match locale {
        &SupportedLocale::EnUs => ALPHABET_EN.chars(),
        _ => ALPHABET_EN.chars(),
    }
}
