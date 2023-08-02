use std::path::{PathBuf, Path};

pub struct Script {
    pub path: PathBuf,
}

impl Script {
    pub fn new<P: AsRef<Path>>(path: P) -> Script {
        Script { 
            path: path.as_ref().to_owned()
        }
    }
}