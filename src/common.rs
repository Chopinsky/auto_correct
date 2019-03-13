#![allow(unreachable_patterns)]

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::str::Chars;
use std::cmp::Ordering;

use crossbeam_channel as channel;
use hashbrown::HashMap;
use crate::SupportedLocale;
use crate::candidate::Candidate;
use crate::config::{AutoCorrectConfig, Config};
use crate::AutoCorrect;
use crate::support::en_us;

pub static DELIM: &'static str = ",";
pub static DEFAULT_LOCALE: &'static str = "en-us";

pub(crate) fn ins_repl(
    word: &str,
    set: &HashMap<String, u32>,
    current_edit: u8,
    tx_curr: channel::Sender<Candidate>,
    tx_next: Option<channel::Sender<String>>
) {
    let size = word.len();
    if size == 0 {
        return;
    }

    let len = en_us::ALPHABET.len();

    for idx in 0..len {
        let rune = en_us::ALPHABET[idx];
        let rune_code = get_char_code(rune, 0);

        for pos in 0..=size {
            if pos > 0 {
                let (left, right) = {
                    if pos < size {
                        // 0 < pos < size
                        (&word[0..pos], &word[pos..size])
                    } else {
                        // pos == size
                        (word, "")
                    }
                };

                if pos == 1 && rune.cmp(left) == Ordering::Equal {
                    // insert at pos 0 has already handled this case
                    continue;
                }

                if size > 2 && pos == size - 1 && rune.cmp(right) == Ordering::Equal {
                    // insert at pos (size - 1) has already handled this case
                    continue;
                }

                // insert
                send_one([left, rune, right].join(""),
                         current_edit, set, &tx_curr, &tx_next);

                // replace
                if rune_code != get_char_code(word, pos - 1) {
                    send_one([&left[..pos - 1], rune, right].join(""),
                             current_edit, set, &tx_curr, &tx_next);
                }
            } else {
                // if pos == 0, just insert
                send_one([rune, word].join(""),
                         current_edit, set, &tx_curr, &tx_next);
            }
        }
    }
}

pub(crate) fn del_tran(
    word: &str,
    set: &HashMap<String, u32>,
    current_edit: u8,
    tx_curr: channel::Sender<Candidate>,
    tx_next: Option<channel::Sender<String>>
) {
    let size = word.len();
    if size <= 1 {
        return;
    }

    for pos in 1..=size {
        let (left, del, right) =
            if pos < size {
                (&word[..pos - 1], &word[pos - 1..pos], &word[pos..])
            } else {
                (&word[..size - 1], &word[size - 1..size], "")
            };

        if pos < size && del.cmp(&right[..1]) == Ordering::Equal{
            continue;
        }

        // delete
        send_one([left, right].join(""),
                 current_edit, set, &tx_curr, &tx_next);

        // transpose
        if pos < size {
            send_one([left, &right[..1], del, &right[1..]].join(""),
                     current_edit, set, &tx_curr, &tx_next);
        }
    }
}

fn send_one(
    target: String,
    edit: u8,
    set: &HashMap<String, u32>,
    store: &channel::Sender<Candidate>,
    tx_next: &Option<channel::Sender<String>>
) {
    if let Some(next_chan) = tx_next {
        next_chan
            .send(target.clone())
            .unwrap_or_else(|err| {
                eprintln!("Failed to search the string: {:?}", err);
            });
    }

    if set.contains_key(&target) {
        let score = set[&target];
        store
            .send(Candidate::new(target, score, edit))
            .unwrap_or_else(|err| {
                eprintln!("Failed to search the string: {:?}", err);
            });
    }
}

pub(crate) fn generate_reverse_dict(config: &Config) -> HashMap<String, Vec<String>> {
    let mut result: HashMap<String, Vec<String>> = HashMap::new();

    // one worker to read from file
    let (tx, rx) = channel::unbounded();
    let dict_path = config.get_dict_path();
    let locale = config.get_locale();

    AutoCorrect::run_job(move || {
        load_dict_async(dict_path, tx);
    });

    // one worker to write to memory
    for word in rx {
        for variation in find_variations(word.clone(), locale) {
            update_reverse_dict(word.clone(), variation, &mut result);
        }
    }

    result
}

pub(crate) fn find_variations(
    word: String,
    locale: SupportedLocale,
) -> channel::Receiver<String>
{
    let (tx, rx) = channel::unbounded();

    AutoCorrect::run_job(move || {
        let len = word.len() + 1;
        for pos in 0..len {
            variations_at_pos(word.clone(), pos, len, locale, &tx);
        }
    });

    rx
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
            tx.send(line).expect("Failed to load the dictionary...");
        }
    }
}

fn get_char_code(word: &str, pos: usize) -> &u8 {
    match word.as_bytes().get(pos) {
        Some(res) => res,
        None => &0u8,
    }
}

fn variations_at_pos(
    word: String,
    pos: usize,
    len: usize,
    locale: SupportedLocale,
    tx: &channel::Sender<String>
)
{
    if pos >= len {
        return;
    }

    let mut remove_base = word.clone();
    let mut removed = '\u{0000}';

    if pos < len - 1 && remove_base.len() > 1 {
        // deletes
        removed = remove_base.remove(pos);
        tx.send(remove_base.clone()).expect("Failed to send the search result...");
    }

    for rune in get_char_set(locale) {
        // inserts
        let mut base = word.clone();
        base.insert(pos, rune);
        tx.send(base).expect("Failed to send the search result...");

        // replaces if we've actually removed a char
        if removed != '\u{0000}' && rune != removed {
            let mut replace = remove_base.clone();
            replace.insert(pos, rune);
            tx.send(replace).expect("Failed to send the search result...");
        }
    }

    // transpose: if we've removed
    if removed != '\u{0000}' && pos > 0 {
        remove_base.insert(pos - 1, removed);
        tx.send(remove_base.clone()).expect("Failed to send the search result...");
    }
}

fn get_char_set(locale: SupportedLocale) -> Chars<'static> {
    match locale {
        SupportedLocale::EnUs => en_us::ALPHABET_EN.chars(),
        _ => en_us::ALPHABET_EN.chars(),
    }
}

fn update_reverse_dict(word: String, variation: String, dict: &mut HashMap<String, Vec<String>>) {
    if let Some(vec) = dict.get_mut(&variation) {
        if !vec.contains(&word) {
            vec.push(word);
        }

        return;
    }

    dict.insert(variation, vec![word]);
}

mod deprecated {
    use crossbeam_channel as channel;
    use hashbrown::HashMap;
    use crate::candidate::Candidate;
    use crate::support::en_us;

    fn delete_n_replace(
        word: String,
        set: &HashMap<String, u32>,
        current_edit: u8,
        tx_curr: channel::Sender<Candidate>,
        tx_next: Option<channel::Sender<String>>,
    )
    {
        let mut base: String;
        let mut replace: String;
        let mut removed: char = '\u{0001}';
        let mut last_removed: char;

        // deletes
        for pos in 0..word.len() {
            base = word.clone();

            last_removed = removed;
            removed = base.remove(pos);

            if tx_next.is_some() && last_removed != removed {
                send_next_string(base.clone(), &tx_next);
            }

            // replaces
            for rune in en_us::ALPHABET_EN.chars() {
                if rune == removed {
                    continue;
                }

                replace = base.clone();
                replace.insert(pos, rune);

                if tx_next.is_some() {
                    send_next_string(replace.clone(), &tx_next);
                }

                if set.contains_key(&replace) {
                    let score = set[&replace];
                    send_candidate(Candidate::new(replace, score, current_edit), &tx_curr);
                }
            }

            if set.contains_key(&base) {
                let score = set[&base];
                send_candidate(Candidate::new(base, score, current_edit), &tx_curr);
            }
        }
    }

    fn transpose_n_insert(
        word: String,
        set: &HashMap<String, u32>,
        current_edit: u8,
        tx_curr: channel::Sender<Candidate>,
        tx_next: Option<channel::Sender<String>>,
    )
    {
        let edit_two = tx_next.is_some();

        let mut base: String;
        let mut removed: char = '\u{0001}';
        let mut last_removed: char;

        // transposes
        let len = word.len() + 1;
        for pos in 0..len {
            if pos > 0 && pos < len - 1 {
                base = word.clone();

                last_removed = removed;
                removed = base.remove(pos);

                if last_removed == removed {
                    continue;
                }

                base.insert(pos - 1, removed);

                if edit_two && !base.is_empty() {
                    send_next_string(base.clone(), &tx_next);
                }

                if set.contains_key(&base) {
                    let score = set[&base];
                    send_candidate(Candidate::new(base, score, current_edit), &tx_curr);
                }
            }

            // inserts
            for rune in en_us::ALPHABET_EN.chars() {
                base = word.clone();
                base.insert(pos, rune);

                if edit_two {
                    send_next_string(base.clone(), &tx_next);
                }

                if set.contains_key(&base) {
                    let score = set[&base];
                    send_candidate(Candidate::new(base, score, current_edit), &tx_curr);
                }
            }
        }
    }

    fn send_candidate(candidate: Candidate, tx: &channel::Sender<Candidate>,) {
        tx.send(candidate).expect("Failed to return a candidate");
    }

    fn send_next_string(word: String, tx: &Option<channel::Sender<String>>) {
        if let Some(tx_next) = tx {
            tx_next.send(word).unwrap_or_else(|err| {
                eprintln!("Failed to search the string: {:?}", err);
            });
        }
    }
}