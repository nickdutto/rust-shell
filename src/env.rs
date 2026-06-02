use std::env;
use std::path::PathBuf;

pub fn get_env_paths(path_env_var: &str) -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = vec![];
    match env::var_os(path_env_var) {
        Some(var_paths) => {
            for path in env::split_paths(&var_paths) {
                if path.is_dir() {
                    paths.push(path.to_path_buf());
                }
            }

            Ok(paths)
        }
        None => Err(format!(
            "{} is not defined in the environment.",
            path_env_var
        )),
    }
}
