use ::Result;

use std::fs;
use std::path::PathBuf;

pub mod loader;
pub use self::loader::{load_all_scripts,
                       load_default_scripts,
                       load_private_scripts};

#[derive(Debug)]
pub struct Script {
}

impl Script {
    pub fn load_from_path(path: PathBuf) -> Result<Script> {
        info!("loading script from: {:?}", path);
        let code = fs::read_to_string(path)?;
        let script = Script::load(code);
        Ok(script)
    }

    pub fn load(_code: String) -> Script {
        // unimplemented!()
        Script { }
    }
}
