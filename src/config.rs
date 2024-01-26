use std::{ffi::CString, sync::OnceLock};

#[derive(Debug)]
pub struct Config {
    pub skip_logo: bool,
    pub log_msgbox: bool,
    pub pdb_file: Option<CString>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skip_logo: true,
            log_msgbox: true,
            pdb_file: Some(CString::new("MapleStory.pdb").unwrap())
        }
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();