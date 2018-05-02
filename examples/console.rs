extern crate auto_correct;

use std::io;
use auto_correct::*;

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

                let check = input.clone();
                let result = correct_service.candidates(check);
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