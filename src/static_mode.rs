use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use super::{AutoCorrect, SupportedLocale};
use candidate::Candidate;
use common::*;
use config::{AutoCorrectConfig, Config};
use crossbeam_channel as channel;
use threads_pool::*;

pub(crate) fn initialize(service: &AutoCorrect) {

}

//TODO: only need table for 1 edit-distance, all ensuing distance can be stacked upon the previous layer