extern crate auto_correct;

use std::io;
use std::ops::Div;
use std::time::{SystemTime};
use auto_correct::prelude::*;

static OPT: &'static str = "OPT";
static EXIT: &'static str = "EXIT";

fn main() {
    let correct_service = AutoCorrect::new();
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

                //TODO: check word correction here
                println!("Input as: {}\n", input);

                let mut result: Vec<Candidate> = Vec::new();
                let now = SystemTime::now();

                // run multiple times to benchmark
                for _i in 0..20 {
                    let check = input.clone();
                    result = correct_service.candidates(check);
                }

                if let Ok(t) = now.elapsed() {
                    println!("Time elapsed: {:?}", t.div(20));
                }

                println!("Output as: {:?}\n", result);

                input.clear();
            },
            Err(error) => {
                println!("error: {}", error);
                break;
            },
        }
    }
}