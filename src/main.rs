#[allow(unused_imports)]
use std::io::{self, Write};
use std::io::Read;

fn main() {
    print!("$ ");
    io::stdout().flush().unwrap();

    let mut command = String::new();
     io::stdin().read_line(&mut command).unwrap();

    println!("{}: command not found", command.trim());
}
