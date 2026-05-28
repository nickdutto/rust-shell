mod command;
mod shell;

use crate::shell::Shell;

fn main() {
    Shell::start_session();
}

// struct ShellCommand;
//
// impl ShellCommand {
//     fn run(input: &str) {
//         let command = input.trim();
//         match command {
//             "exit" => Self::run_exit(),
//             s if s.starts_with("echo") => Self::run_echo(input),
//             s if s.starts_with("type") => Self::run_type(input),
//             _ => Self::run_executable(input),
//         }
//     }
//
//     fn run_exit() {
//         io::stdout().flush().unwrap();
//         std::process::exit(1);
//     }
//
//     fn run_echo(input: &str) {
//         println!("{}", input[5..].trim());
//     }
//
//     fn run_type(input: &str) {
//         let builtin_commands = ["echo", "exit", "type"];
//         let command = input[5..].trim().to_string();
//
//         if builtin_commands.contains(&command.as_str()) {
//             println!("{} is a shell builtin", command);
//         } else {
//             let command_in_path = run_command_by_path(&command, PathCommandMode::Type);
//
//             if !command_in_path {
//                 println!("{}: not found", command);
//             }
//         }
//     }
//
//     fn run_executable(input: &str) {
//         let command_in_path = run_command_by_path(input, PathCommandMode::Execute);
//
//         if !command_in_path {
//             println!("{}: command not found", input);
//         }
//     }
// }

// struct Shell;
//
// impl Shell {
//     fn start_session() {
//         loop {
//             print!("$ ");
//             io::stdout().flush().unwrap();
//
//             let mut command = String::new();
//             io::stdin().read_line(&mut command).unwrap();
//             ShellCommand::run(&command);
//         }
//     }
// }
