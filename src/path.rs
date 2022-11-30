use crate::{error::Error, Result};
use std::{env, path::PathBuf};

#[cfg(unix)]
const PATH_SEPARATOR: char = '/';

#[cfg(windows)]
const PATH_SEPARATOR: char = '\\';

/// Absolutize path. Expand:
/// - `~`
/// - environment variables
/// - `.` and `..`
pub fn absolutize_path(path: &str) -> Result<PathBuf> {
    // Expand `~` to value of `HOME` env variable, if present.
    let mut path = path.replace('~', &env::var("HOME").map_err(|_| Error::HomeNotFound)?);

    // Expand environment variables.
    if path.contains('$') {
        let mut vars: Vec<String> = vec![];

        // Find environment variables in `path`.
        let mut var = String::default();
        let chars: Vec<char> = path.chars().collect();
        let mut i = 0;
        while i < path.len() {
            if chars[i] == '$' {
                while chars[i] != PATH_SEPARATOR {
                    var.push(chars[i]);
                    i += 1;
                }
                vars.push(var.clone());
                var.clear();
            }
            i += 1;
        }

        // Expand found environment variables.
        for var in vars {
            path = path.replace(
                &var,
                &env::var(&var[1..]).map_err(|err| Error::EnvVar(var.clone(), err))?,
            );
        }
    }

    Ok(PathBuf::from(&path))
}
