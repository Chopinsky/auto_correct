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

use candidate::Candidate;
use threads_pool::*;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses
//TODO: customizable score function

static MAX_EDIT: u8 = 2;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SupportedLocale {
    EnUs,
}

pub struct AutoCorrect {
    pub max_edit: u8,
    pool: ThreadPool,
    locale: SupportedLocale,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_locale(SupportedLocale::EnUs)
    }

    // TODO: make this public when more locale dict are added
    fn new_with_locale(locale: SupportedLocale) -> AutoCorrect {
        // max edit only allowed between 1 and 3
        let max_edit = if MAX_EDIT > 3 {
            3
        } else if MAX_EDIT < 1 {
            1
        } else {
            MAX_EDIT
        };

        let pool = ThreadPool::new(2);

        let service = AutoCorrect {
            max_edit,
            pool,
            locale,
        };

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
        dynamic_mode::initialize(&service);

        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        dynamic_mode::candidate(self.locale.clone(), word, 0, self.max_edit, &self.pool)
    }
}
