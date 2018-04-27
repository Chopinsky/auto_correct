#[macro_use]
extern crate lazy_static;

use std::collections::HashMap;
use std::sync::{Once, ONCE_INIT, RwLock};

lazy_static! {
    static ref WORDS_SET: RwLock<Box<HashMap<String, u32>>> = RwLock::new(Box::new(HashMap::new()));
    static ref INIT: RwLock<bool> = RwLock::new(false);
}

static DICTIONARY_PATH: &'static str = "../resources/words.txt";
static LAUNCH: Once = ONCE_INIT;

//TODO: define config struct -- 1. memory mode vs. speed mode; 2. one miss vs. two misses

pub fn initialize() {
    // if already initialized
    LAUNCH.call_once(|| {
        if let Ok(mut init) = INIT.write() {
            *init = true;
        }

        //TODO: load words into the hashset
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_test() {
        initialize();

        let init = *INIT.read().unwrap();
        assert!(init);

        let size = WORDS_SET.read().unwrap().len();
        assert_eq!(size, 0); // TODO: use correct size
    }
}
