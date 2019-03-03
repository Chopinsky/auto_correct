use crate::support::en_us;
use std::time::{SystemTime};

fn enum_print(word: &str, pos: usize) -> Vec<String> {
    let left = &word[..pos];
    let right = &word[pos..];
    let len = en_us::ALPHABET.len();

    let mut vec = Vec::with_capacity(len);
    (0..len).for_each(|idx| {
        let res = [left, en_us::ALPHABET[idx], right].join("");
        vec.push(res);
    });

    vec
}