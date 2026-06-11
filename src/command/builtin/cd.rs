use crate::io::tokenize::Tokens;
use crate::io::writer::{OutputType, write_output};
use std::env;
use std::io::ErrorKind;
use std::path::Path;

pub fn handle_cd(tokens: Tokens) {
    let target = tokens
        .arguments
        .first()
        .map(|s| s.as_str().trim())
        .unwrap_or("~");

    let result = match target {
        "~" => {
            if let Some(home) = env::var_os("HOME") {
                cd_set_dir(Path::new(&home))
            } else {
                Ok(())
            }
        }
        _ => cd_set_dir(Path::new(&target)),
    };

    if let Err(err_message) = result {
        write_output(err_message.trim(), OutputType::Stderr, Some(&tokens));
    }
}

fn cd_set_dir(path: &Path) -> Result<(), String> {
    match env::set_current_dir(path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == ErrorKind::NotFound => Err(format!(
            "cd: {}: No such file or directory",
            path.to_str().unwrap()
        )),
        Err(e) => Err(format!("cd: {}: {}", path.display(), e)),
    }
}
