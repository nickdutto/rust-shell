use is_executable::is_executable;
use std::env;
use std::path::PathBuf;

pub fn get_env_paths(path_env_var: &str) -> Result<Vec<PathBuf>, String> {
    let mut paths: Vec<PathBuf> = vec![];
    match env::var_os(path_env_var) {
        Some(var_paths) => {
            for path in env::split_paths(&var_paths) {
                if path.is_dir() {
                    paths.push(path.clone());
                }
            }

            Ok(paths)
        }
        None => Err(format!("{path_env_var} is not defined in the environment.")),
    }
}

pub fn get_env_path_executables(path_env_var: &str) -> Vec<String> {
    let path_var = if cfg!(debug_assertions) {
        "RUST_SHELL_DEBUG_PATH"
    } else {
        path_env_var
    };

    let Ok(paths) = get_env_paths(path_var) else {
        return vec![];
    };

    let mut executables: Vec<String> = paths
        .into_iter()
        .filter_map(|dir_path| dir_path.read_dir().ok())
        .flatten()
        .filter_map(Result::ok)
        .filter(|entry| {
            if let Ok(file_type) = entry.file_type() {
                file_type.is_file() || file_type.is_symlink()
            } else {
                false
            }
        })
        .filter(|entry| is_executable(entry.path()))
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();

    executables.sort();
    executables.dedup();

    executables
}
