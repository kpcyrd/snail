use scripts::Script;
use ::Result;

use std::fs;
use std::env;
use std::path::Path;


pub fn load_all_scripts() -> Result<Vec<Script>> {
    let mut scripts = Vec::new();
    scripts.extend(load_default_scripts()?);
    scripts.extend(load_private_scripts()?);
    Ok(scripts)
}

pub fn load_default_scripts() -> Result<Vec<Script>> {
    load_from_folder("/usr/lib/snaild/scripts")
}

pub fn load_private_scripts() -> Result<Vec<Script>> {
    match env::home_dir() {
        Some(home) => load_from_folder(home.join(".config/snaild/scripts/")),
        None => Ok(Vec::new()),
    }
}

pub fn load_from_folder<P: AsRef<Path>>(path: P) -> Result<Vec<Script>> {
    match fs::read_dir(path) {
        Ok(paths) => paths
                        .map(|x| Script::load_from_path(x?.path()))
                        .collect(),
        // if this fails, ignore the error
        Err(_) => Ok(Vec::new()),
    }
}
