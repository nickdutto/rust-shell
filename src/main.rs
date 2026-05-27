#[allow(unused_imports)]
use std::io::{self, Write};
use std::process::exit;

fn main() {
    let builtin_commands = vec!["echo", "exit", "type"];

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
            for builtin_command in &builtin_commands {
                if *builtin_command == command {
                    println!("{} is a shell builtin", builtin_command);
                    break;
                }
            }
        } else {
            println!("{}: command not found", command);
        }
    }
}
