use super::{AutoCorrect};
use config::{Config};
use threads_pool::*;

pub(crate) fn initialize(service: &AutoCorrect) {
    // if already initialized, calling this function takes no effect
    if let Err(e) = populate_words_set(&service.config, &service.pool) {
        eprintln!("Failed to initialize: {}", e);
        return;
    }
}

fn populate_words_set(_config: &Config, _pool: &ThreadPool) -> Result<(), String> {
    Ok(())
}

//TODO: only need table for 1 edit-distance, all ensuing distance can be stacked upon the previous layer