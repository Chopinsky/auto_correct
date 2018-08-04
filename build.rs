#![allow(dead_code)]

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

static ALPHABET_EN: &'static str = "abcdefghijklmnopqrstuvwxyz";

fn find_all_variations(word: String) -> Vec<String> {
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

fn main() {
    let skip_rebuild =
        if let Ok(result) = env::var("SKIP_DICT_REBUILD") {
            result.to_lowercase()
        } else {
            String::new()
        };

    if skip_rebuild.len() > 0 && skip_rebuild != String::from("true") {
        let out_dir =
            if let Ok(result) = env::var("LOCALE") {
                format!("./resources/{}/", result.to_lowercase())
            } else {
                format!("./resources/{}/", String::from("en-us"))
            };

        let dest_path = Path::new(&out_dir).join("freq_50k_precalc.txt");
        let mut f = File::create(&dest_path).unwrap();

        //    f.write_all(b"
        //        pub fn message() -> &'static str {
        //            \"Hello, World!\"
        //        }
        //    ").unwrap();
    }
}
