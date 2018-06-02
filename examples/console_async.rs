extern crate auto_correct;

use std::io;
use std::ops::Div;
use std::sync::mpsc;
use std::time::{SystemTime};
use auto_correct::prelude::*;

static OPT: &'static str = "OPT";
static EXIT: &'static str = "EXIT";

fn main() {
    let correct_service = AutoCorrect::new_with_max_edit(1);
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

                // run multiple times to benchmark
                let check = input.clone();
                let (tx, rx) = mpsc::channel();

                let now = SystemTime::now();
                correct_service.candidates_async(check, tx);

                let mut count = 5;
                for result in rx {
                    println!("Suggestion: {}; Score: {}; Edit Distance: {}",
                             result.word, result.score, result.edit);

                    count -= 1;
                    if count == 0 {
                        break;
                    }
                }

                if let Ok(t) = now.elapsed() {
                    println!("\nTime elapsed: {:?}\n====================\n", t.div(1));
                }

                input.clear();
            },
            Err(error) => {
                println!("error: {}", error);
                break;
            },
        }
    }
}