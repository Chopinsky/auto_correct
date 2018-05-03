
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{PathBuf};
use std::sync::mpsc;

use candidate::Candidate;

pub static ALPHABET: &'static str = "abcdefghijklmnopqrstuvwxyz";
pub static DICTIONARY_PATH: &'static str = "./resources/words2.txt";
pub static DELIM: &'static str = "^";

pub fn send_one_candidate(word: String, edit: u8, set: &Box<HashMap<String, u32>>, tx: &mpsc::Sender<Candidate>) {
    let score = set[&word];
    tx.send(Candidate::new(word, score, edit)).expect("Failed to send the candidate to the caller");
}

pub fn open_file_async(path: &str, tx: mpsc::Sender<String>) {
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