#![allow(dead_code)]

//! This library provides auto-correct suggestions that are within 1 edit distance from
//! known English words.

#[macro_use]
extern crate lazy_static;
extern crate threads_pool;

mod candidate;
mod common;
mod dynamic_mode;

pub mod prelude {
    pub use AutoCorrect;
    pub use candidate::Candidate;
}

use threads_pool::*;
use candidate::Candidate;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses
//TODO: customizable score function

pub struct AutoCorrect {
    pub max_edit: u8,
    pool: ThreadPool,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        let pool = ThreadPool::new(2);

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
        dynamic_mode::initialize(&pool);

        AutoCorrect { pool, max_edit: 1 }
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        dynamic_mode::candidate(word, 0, self.max_edit, &self.pool)
    }
}
