use crate::io::stream::IoStreams;
use crate::io::tokenize::Tokens;
use std::env;
use std::io::ErrorKind;
use std::io::Write;
use std::path::Path;

pub fn handle_cd(tokens: Tokens, mut io_streams: IoStreams) {
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

    if let Err(e) = result {
        writeln!(io_streams.error, "{}", e.trim()).unwrap();
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
