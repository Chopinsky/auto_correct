[Auto_Correct] [docsrs]
======================

[![Rusty_Express on crates.io][cratesio-image]][cratesio]
[![Rusty_Express on docs.rs][docsrs-image]][docsrs]

[cratesio]: https://crates.io/crates/auto_correct
[cratesio-image]: https://img.shields.io/crates/v/auto_correct.svg
[docsrs-image]: https://docs.rs/auto_correct/badge.svg
[docsrs]: https://docs.rs/auto_correct

## What is this
This library provides the service to suggest auto-corrections on words within 1 ~ 3 edit distances, based on configurations.

Currently the project only supports English words corrections, and we plan to expand the service to more languages.

## How to use
In your project's `Cargo.toml`, add dependency:
```cargo
[dependencies]
auto_correct = "^0.1.0"
...
```

In `src\main.rs`:
```rust
extern crate auto_correct;

use auto_correct::prelude::*;

fn main() {
    // Initialize the service
    let correct_service = AutoCorrect::new();

    // Vector `results` contains an array of the `Candidate` objects, which is sorted by scores
    let results = correct_service.candidates("wodr");

    // Print out the result to the screen
    for idx in 0..results.len() {
        println!("Suggestion #{}: {}; Score: {}; Edit Distance: {}",
                    idx, results[idx].word, results[idx].score, results[idx].edit);
    }
}
```

Alternatively, if you would want a more responsive approach, you can use the `candidate_async` function to get the corrections:
```rust
extern crate auto_correct;

use std::sync::mpsc;
use auto_correct::prelude::*;

fn main() {
    // Initialize the service
    let correct_service = AutoCorrect::new();
    let (tx, rx) = mpsc::channel();

    // Vector `results` contains an array of the `Candidate` objects, which is sorted by scores
    correct_service.candidates_async("wodr", tx);

    // Print out the result to the screen when receiving new suggestions.
    for result in rx {
        println!("Suggestion: {}; Score: {}; Edit Distance: {}",
                 result.word, result.score, result.edit);
    }
}
```
