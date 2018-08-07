#![allow(dead_code)]

extern crate flate2;

use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::thread;

static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";

fn main() {
    let skip_rebuild =
        if let Ok(result) = env::var("SKIP_DICT_REBUILD") {
            result.to_lowercase()
        } else {
            String::new()
        };

    if skip_rebuild != String::from("true") {
        let out_dir =
            if let Ok(result) = env::var("LOCALE") {
                format!("./resources/{}/", result.to_lowercase())
            } else {
                format!("./resources/{}/", String::from("en-us"))
            };

        let dest_path = Path::new(&out_dir).join("freq_50k_precalc.txt");
        let mut f = File::create(&dest_path).unwrap();

        refresh_dict(&out_dir, &mut f);
    }
}

fn find_variations(word: String) -> Vec<String> {
    let mut result: Vec<String> = Vec::new();
    let mut base: String;

    let mut replace: String;
    let mut removed: char;

    let len = word.len() + 1;
    for pos in 0..len {
        if pos < len - 1 {
            base = word.clone();

            // deletes
            removed = base.remove(pos);
            result.push(base.clone());

            // replaces
            for chara in ALPHABET_EN.chars() {
                if chara == removed {
                    continue;
                }

                replace = base.clone();
                replace.insert(pos, chara);

                result.push(replace.clone());
            }

            // transposes
            if pos > 0 {
                base.insert(pos - 1, removed);
                result.push(base.clone());
            }
        }

        // inserts
        for chara in ALPHABET_EN.chars() {
            base = word.clone();
            base.insert(pos, chara);

            result.push(base.clone());
        }
    }

    result
}

fn refresh_dict(source_dir: &String, target: &mut File) {
    let (tx, rx) = mpsc::channel();
    let source = source_dir.clone();

    thread::spawn(move || {
        let path =
            if let Ok(override_dict) = env::var("OVERRIDE_DICT") {
                PathBuf::from(&override_dict)
            } else {
                Path::new(&source).join("freq_50k.txt")
            };

        if !path.exists() || !path.is_file() {
            eprintln!("Unable to open the source dictionary from path: {:?}...", path);
            return;
        }

        let file = File::open(path).expect("file not found");
        let reader = BufReader::new(file);

        for raw_line in reader.lines() {
            if let Ok(line) = raw_line {
                tx.send(line).unwrap();
            }
        }
    });

    let mut key: String;
    let mut result: String;

    for entry in rx {
        let temp: Vec<&str> = entry.splitn(2, ",").collect();
        if temp[0].is_empty() {
            continue;
        }

        key = temp[0].to_owned();
        result = format!("{}\n", key);

        target.write(result.as_bytes()).unwrap();
    }
}