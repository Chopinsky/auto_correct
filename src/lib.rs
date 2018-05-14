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

use std::sync::mpsc;
use candidate::Candidate;
use threads_pool::*;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses
//TODO: customizable score function

static DEFAULT_MAX_EDIT: u8 = 1;
static MAX_EDIT_THRESHOLD: u8 = 3;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SupportedLocale {
    EnUs,
}

pub struct AutoCorrect {
    max_edit: u8,
    pool: ThreadPool,
    locale: SupportedLocale,
}

impl AutoCorrect {
    pub fn new() -> AutoCorrect {
        AutoCorrect::new_with_locale(SupportedLocale::EnUs)
    }

    // TODO: make this public when more locale dict are added
    fn new_with_locale(locale: SupportedLocale) -> AutoCorrect {
        let service = AutoCorrect {
            max_edit: DEFAULT_MAX_EDIT,
            pool: ThreadPool::new(2),
            locale,
        };

        //TODO: if speed mode, also load the variation1 (and variation 2 if allowing 2 misses)
        dynamic_mode::initialize(&service);

        service
    }

    pub fn candidates(&self, word: String) -> Vec<Candidate> {
        dynamic_mode::candidate(self.locale.clone(), word, 0, self.max_edit, &self.pool)
    }

    pub fn candidates_async(&self, word: String, tx: mpsc::Sender<Candidate>) {
        //TODO: async mode -- send Candidate once found one
    }

    pub fn set_max_edit(&mut self, max_edit: u8) {
        // max edit only allowed between 1 and 3
        self.max_edit = if max_edit > MAX_EDIT_THRESHOLD {
            eprintln!("Only support max edits less or equal to {}.", MAX_EDIT_THRESHOLD);
            3
        } else if max_edit < 1 {
            eprintln!("Only support max edits greater or equal to 1.");
            1
        } else {
            max_edit
        };
    }

    #[inline]
    pub fn get_max_edit(&self) -> u8 {
        self.max_edit
    }

    #[inline]
    pub fn get_locale_in_use(&self) -> SupportedLocale {
        self.locale.clone()
    }
}
