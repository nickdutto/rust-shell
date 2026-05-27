#[allow(unused_imports)]
use std::io::{self, Write};

fn main() {
    let builtin_commands = ["echo", "exit", "type"];

    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        let mut command = String::new();
        io::stdin().read_line(&mut command).unwrap();

        command = command.trim().to_string();
        if command == "exit" {
            break;
        } else if command.starts_with("echo") {
            command = command[5..].trim().to_string();
            println!("{}", command);
        } else if command.starts_with("type") {
            command = command[5..].trim().to_string();
            if builtin_commands.contains(&command.as_str()) {
                println!("{} is a shell builtin", command);
            } else {
                println!("{}: not found", command);
                continue;
            }
        } else {
            println!("{}: command not found", command);
        }
    }
}
