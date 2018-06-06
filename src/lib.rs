#![allow(dead_code)]

//! This library provides auto-correct suggestions that are within 1 edit distance from
//! known English words.

#[macro_use]
extern crate lazy_static;
extern crate threads_pool;

mod candidate;
mod common;
mod config;
mod dynamic_mode;

pub mod prelude {
    pub use AutoCorrect;
    pub use config::{AutoCorrectConfig, Config, SupportedLocale};
    pub use candidate::Candidate;
}

use std::sync::{mpsc, Arc};
use candidate::Candidate;
use config::{Config, SupportedLocale};
use threads_pool::*;

//TODO: define config struct -- 1. memory mode vs. speed mode;
//TODO: customizable score function

pub struct AutoCorrect {
    pub config: Config,
    pool: Arc<ThreadPool>,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_config(Config::new())
    }

    fn new_with_config(config: Config) -> AutoCorrect {
        let service = AutoCorrect {
            config,
            pool: Arc::new(ThreadPool::new(2)),
        };

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
        dynamic_mode::initialize(&service);

        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        dynamic_mode::candidate(
            word,
            0,
            &self.config,
            Arc::clone(&self.pool),
            None)
    }

    pub fn candidates_async(&self, word: String, tx: mpsc::Sender<Candidate>) {
        let config_clone = self.config.clone();
        let pool_arc = Arc::clone(&self.pool);

        let (tx_cache, rx_cache) = mpsc::channel();
        self.pool.execute(move || {
            dynamic_mode::candidate(
                word,
                0,
                &config_clone,
                pool_arc,
                Some(tx_cache));
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

    #[inline]
    pub fn get_config<'a>(&'a self) -> &'a Config {
        &self.config
    }
}