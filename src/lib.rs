#![allow(dead_code)]

//! This library provides auto-correct suggestions that are within 1 edit distance from
//! known English words.

#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate threads_pool;
extern crate fst;

mod candidate;
mod common;
mod config;
mod dynamic_mode;
mod hybrid_mode;

pub mod prelude {
    pub use candidate::Candidate;
    pub use config::{AutoCorrectConfig, Config, SupportedLocale};
    pub use AutoCorrect;
}

use candidate::Candidate;
use config::{AutoCorrectConfig, Config, RunMode, SupportedLocale};
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
    #[inline]
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_config(Config::new())
    }

    pub fn new_with_config(config: Config) -> AutoCorrect {
        let service = AutoCorrect {
            config,
            pool: Arc::new(ThreadPool::new(4)),
        };

        service.init_dict();
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

    fn init_dict(&self) {
        match self.config.get_run_mode() {
            RunMode::SpeedSensitive => hybrid_mode::initialize(&self),
            RunMode::SpaceSensitive => dynamic_mode::initialize(&self),
        }
    }
}

impl AutoCorrectConfig for AutoCorrect {
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

    fn set_locale(&mut self, locale: SupportedLocale) {
        if locale == self.config.get_locale() {
            return;
        }

        self.config.set_locale(locale);

        if !self.config.get_override_dict().is_empty() {
            self.init_dict();
        }
    }

    #[inline]
    fn get_locale(&self) -> SupportedLocale {
        self.config.get_locale()
    }

    fn set_run_mode(&mut self, mode: RunMode) {
        if self.config.get_run_mode() == mode {
            return;
        }

        self.config.set_run_mode(mode);
        self.init_dict();
    }

    #[inline]
    fn get_run_mode(&self) -> RunMode {
        self.config.get_run_mode()
    }

    fn set_override_dict(&mut self, dict_path: &str) {
        if dict_path == self.config.get_dict_path() {
            return;
        }

        self.config.set_override_dict(dict_path);
        self.init_dict();

        if self.config.get_run_mode() == RunMode::SpeedSensitive {
            if let Err(e) = self.refresh_hybrid_dict(None) {
                eprintln!("Encountering error while updating the override dict: {}", e);
            }
        }
    }

    #[inline]
    fn get_override_dict(&self) -> String {
        self.config.get_override_dict()
    }
}

pub trait ServiceUtils {
    fn refresh_hybrid_dict(&self, custom_path: Option<String>) -> Result<(), String>;
}

impl ServiceUtils for AutoCorrect {
    fn refresh_hybrid_dict(&self, _custom_path: Option<String>) -> Result<(), String> {
        let dict =
            common::generate_reverse_dict(&self.config, &self.pool);

        //TODO: now compress and save the result to disk

        if self.config.get_run_mode() == RunMode::SpeedSensitive {
            hybrid_mode::set_reverse_dict(dict);
        }

        Ok(())
    }
}
