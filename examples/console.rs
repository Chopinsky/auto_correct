extern crate auto_correct;

use std::io;
use std::ops::Div;
use std::time::{SystemTime};
use auto_correct::prelude::*;

static OPT: &'static str = "OPT";
static EXIT: &'static str = "EXIT";
static LEN: u32 = 10;

fn main() {
    let mut correct_service = AutoCorrect::new();
    correct_service.set_max_edit(2);

    let stream = io::stdin();
    let mut input = String::new();

    loop {
        println!("Enter the word: ");
        match stream.read_line(&mut input) {
            Ok(_) => {
                input = input.trim().to_string();

                if input.to_uppercase().eq(&OPT.to_owned())
                    || input.to_uppercase().eq(&EXIT.to_owned()) {
                    break;
                }

                println!("\nInput as: {}\n", input);

                let mut results: Vec<Candidate> = Vec::new();
                let now = SystemTime::now();

                // run multiple times to benchmark
                for _ in 0..LEN {
                    let check = input.clone();
                    results = correct_service.candidates(check);
                }

                let e = now.elapsed().unwrap();

                println!("\nResults:\n");
                for idx in 0..results.len() {
                    println!("Suggestion #{}: {}; Score: {}; Edit Distance: {}",
                             idx, results[idx].word, results[idx].score, results[idx].edit);
                }

                println!("\nTime elapsed: {:?}", e.div(LEN));
                println!("\n=========================\n");
                input.clear();
            },
            Err(error) => {
                println!("error: {}", error);
                break;
            },
        }
    }
}