use befuddle::{field::FungeField, BefungeExecution};
use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    let path = &args[1];
    let contents = fs::read_to_string(path).expect("Failed to read program");

    let mut exec = BefungeExecution::new(FungeField::from_str(&contents, 80, 25));

    exec.run_with_terminal();
    println!();
}
