#![allow(dead_code)]

//! This library provides auto-correct suggestions that are within 1 edit distance from
//! known English words.

#[macro_use]
extern crate lazy_static;
extern crate crossbeam_channel;
extern crate threads_pool;
extern crate hashbrown;

mod candidate;
mod common;
mod config;
mod dynamic;
mod hybrid;
mod stores;
mod support;
mod trie;

pub mod prelude {
    pub use candidate::Candidate;
    pub use config::{AutoCorrectConfig, Config, SupportedLocale};
    pub use AutoCorrect;
}

use std::sync::mpsc;
use candidate::Candidate;
use config::{AutoCorrectConfig, Config, RunMode, SupportedLocale};

use crossbeam_channel as channel;
use hashbrown::HashSet;
use threads_pool::ThreadPool;

//TODO: define config struct -- 1. memory mode vs. speed mode;
//TODO: customizable score function

static mut POOL: Option<ThreadPool> = None;
const POOL_SIZE: usize = 8;

pub struct AutoCorrect {
    config: Config,
}

impl AutoCorrect {
    #[inline]
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_config(Config::new())
    }

    pub fn new_with_config(config: Config) -> AutoCorrect {
        let service = AutoCorrect {
            config,
        };

        AutoCorrect::pool_init(POOL_SIZE);
        service.init_dict();

        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        let max_edit = self.config.get_max_edit();
        let locale = self.config.get_locale();

        stores::get_ready();
        let mut result =
            dynamic::candidate(word, 0, max_edit, locale, &mut None, 0);
        stores::reset();

        let mut vec = Vec::with_capacity(result.len());
        result.drain().for_each(|candidate| {
            vec.push(candidate);
        });

        vec.sort_by(|a, b| b.cmp(a));
        vec
    }

    pub fn candidates_async(&self, word: String, tx: mpsc::Sender<Candidate>) {
        let max_edit = self.config.get_max_edit();
        let locale = self.config.get_locale();
        let (tx_cache, rx_cache) = channel::unbounded();

        stores::get_ready();
        AutoCorrect::run_job(move || {
            dynamic::candidate(word, 0, max_edit, locale, &mut Some(tx_cache), 0);
        });

        let mut cache = HashSet::with_capacity(16);
        for result in rx_cache {
            if !cache.contains(&result.word) {
                cache.insert(result.word.clone());

                // send the result back, if the channel is closed, just return.
                if tx.send(result).is_err() {
                    break;
                }
            }
        }

        stores::reset();
    }

    pub(crate) fn run_job<F: FnOnce() + Send + 'static>(f: F) {
        if let Some(pool) = unsafe { POOL.as_mut() } {
            if pool.exec(f, true).is_err() {
                eprintln!("Failed to execute the search service...");
            };

            return;
        }

        eprintln!("Failed to execute the service...");
    }

    fn init_dict(&self) {
        match self.config.get_run_mode() {
            RunMode::SpeedSensitive => hybrid::initialize(&self),
            RunMode::SpaceSensitive => dynamic::initialize(&self),
        }
    }

    fn pool_init(size: usize) {
        unsafe { POOL.replace(ThreadPool::new(size)); }
    }
}

impl Default for AutoCorrect {
    fn default() -> Self {
        Self::new()
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
            common::generate_reverse_dict(&self.config);

        //TODO: now compress and save the result to disk

        if self.config.get_run_mode() == RunMode::SpeedSensitive {
            hybrid::set_reverse_dict(dict);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests_dyn {
    use super::*;

    #[test]
    fn base() {
        let mut service = AutoCorrect::new();
        service.set_max_edit(2);
        assert_eq!(service.candidates(String::from("tets")).len(), 360usize);
    }

    #[test]
    fn long() {
        let mut service = AutoCorrect::new();
        service.set_max_edit(2);
        assert_eq!(service.candidates(String::from("wahtabout")).len(), 1usize);
    }

    #[test]
    fn none() {
        let mut service = AutoCorrect::new();
        service.set_max_edit(2);
        assert!(service.candidates(String::from("whataboutism")).is_empty());
    }
}