extern crate pwhash;

use pwhash::unix;
use std::env;
use std::fs::File;
use std::io::{self, BufRead};
use std::process;

fn main() {
    let args: Vec<String> = env::args().take(3).collect();

    if args.len() != 3 {
        println!("Usage: {} <wordlist file> <password hash>", args[0]);
    }
    let dictionary_path = &args[1];
    let file = File::open(dictionary_path).unwrap_or_else(|e| {
        println!("Error '{}' happened. Exiting.", e);
        process::exit(1);
    });
    for line in io::BufReader::new(file).lines() {
        if let Ok(word) = line {
            if unix::verify(&word, &args[2]) {
                println!("Password '{}' has been found.", word);
                process::exit(0);
            }
        }
    }
}
