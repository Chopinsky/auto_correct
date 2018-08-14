extern crate auto_correct;

use auto_correct::prelude::*;
use auto_correct::ServiceUtils;

fn main() {
    let correct_service = AutoCorrect::new();
    correct_service.refresh_hybrid_dict(None);
}