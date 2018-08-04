extern crate auto_correct;

use std::io;
use auto_correct::prelude::*;
use auto_correct::ServiceUtils;

fn main() {
    ServiceUtils::refresh_hybrid_dict(None);
}