use super::{AutoCorrect};
use std::collections::HashMap;
use std::sync::RwLock;
use config::{Config};
use threads_pool::*;

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
    static ref REVERSE_DICT: RwLock<Box<HashMap<String, Vec<String>>>> = RwLock::new(Box::new(HashMap::new()));
}

pub(crate) fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    if let Err(e) = populate_words_set(&service.config, &service.pool) {
        eprintln!("Failed to initialize: {}", e);
        return;
    }
}

pub(crate) fn set_reverse_dict(dict: HashMap<String, Vec<String>>) {
    if let Ok(mut rev_dict) = REVERSE_DICT.write() {
        *rev_dict = Box::new(dict);
    }
}

fn populate_words_set(_config: &Config, _pool: &ThreadPool) -> Result<(), String> {
    Ok(())
}

//TODO: only need table for 1 edit-distance, all ensuing distance can be stacked upon the previous layer