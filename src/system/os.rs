use std::env;
use std::process::Command;

pub fn get_username() -> Option<String> {
    if let Ok(username) = env::var("USER") {
        Some(username)
    } else {
        let output = Command::new("id").args(["-u", "-n"]).output().ok()?;

        if output.status.success() {
            let username = String::from_utf8(output.stdout).ok()?;
            Some(username)
        } else {
            None
        }
    }
}
