#![allow(dead_code)]

//! This library provides auto-correct suggestions that are within 1 edit distance from
//! known English words.

#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate threads_pool;

mod candidate;
mod common;
mod config;
mod dynamic_mode;

pub mod prelude {
    pub use AutoCorrect;
    pub use candidate::Candidate;
    pub use config::{AutoCorrectConfig, Config, SupportedLocale};
}

use candidate::Candidate;
use config::{AutoCorrectConfig, Config, SupportedLocale};
use crossbeam_channel as channel;
use std::sync::{mpsc, Arc};
use threads_pool::*;

//TODO: define config struct -- 1. memory mode vs. speed mode;
//TODO: customizable score function

pub struct AutoCorrect {
    config: Config,
    pool: Arc<ThreadPool>,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_config(Config::new())
    }

    pub fn new_with_config(config: Config) -> AutoCorrect {
        let service = AutoCorrect {
            config,
            pool: Arc::new(ThreadPool::new(2)),
        };

        service.refresh_dict();
        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        dynamic_mode::candidate(word, 0, &self.config, Arc::clone(&self.pool), None)
    }

    pub fn candidates_async(&self, word: String, tx: mpsc::Sender<Candidate>) {
        let config_clone = self.config.clone();
        let pool_arc = Arc::clone(&self.pool);

        let (tx_cache, rx_cache) = channel::unbounded();
        self.pool.execute(move || {
            dynamic_mode::candidate(word, 0, &config_clone, pool_arc, Some(tx_cache));
        });

        let mut cache = Vec::new();
        for result in rx_cache {
            if !cache.contains(&result.word) {
                cache.push(result.word.clone());

                // send the result back, if the channel is closed, just return.
                if let Err(_) = tx.send(result) {
                    break;
                }
            }
        }
    }

    fn refresh_dict(&self) {
        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
        dynamic_mode::initialize(&self);
    }
}

impl AutoCorrectConfig for AutoCorrect {
    #[inline]
    fn set_max_edit(&mut self, max_edit: u8) {
        if max_edit == self.config.get_max_edit() {
            return;
        }

        self.config.set_max_edit(max_edit);
    }

    #[inline]
    fn get_max_edit(&self) -> u8 {
        self.config.get_max_edit()
    }

    #[inline]
    fn set_locale(&mut self, locale: SupportedLocale) {
        if locale == self.config.get_locale() {
            return;
        }

        self.config.set_locale(locale);

        if !self.config.get_override_dict().is_empty() {
            self.refresh_dict();
        }
    }

    #[inline]
    fn get_locale(&self) -> SupportedLocale {
        self.config.get_locale()
    }

    #[inline]
    fn set_override_dict(&mut self, dict_path: &str) {
        if dict_path == self.config.get_dict_path() {
            return;
        }

        self.config.set_override_dict(dict_path);
        self.refresh_dict();
    }

    #[inline]
    fn get_override_dict(&self) -> String {
        self.config.get_override_dict()
    }
}
