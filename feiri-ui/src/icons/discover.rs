use std::{
    env::{self, var_os},
    path::PathBuf,
};

pub fn data_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Some(data_home) = var_os("XDG_DATA_HOME") {
        if !data_home.is_empty() {
            dirs.push(PathBuf::from(data_home));
        }
    } else if let Some(home) = var_os("HOME") {
        dirs.push(PathBuf::from(home).join(".local/share"));
    }

    let system_dirs = var_os("XDG_DATA_DIRS")
        .filter(|dir| !dir.is_empty())
        .unwrap_or_else(|| "/usr/local/share:/usr/share".into());

    dirs.extend(env::split_paths(&system_dirs));
    dirs
}
