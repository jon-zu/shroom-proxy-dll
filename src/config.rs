use std::{ffi::CString, sync::OnceLock};

#[derive(Debug)]
pub struct AutoLoginData {
    pub username: CString,
    pub password: CString,
    pub world: u32,
    pub channel: u32,
    pub char_index: u32,
}

#[derive(Debug)]
pub struct Config {
    pub skip_logo: bool,
    pub log_msgbox: bool,
    pub pdb_file: Option<CString>,
    pub auto_login_data: Option<AutoLoginData>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            skip_logo: true,
            log_msgbox: false,
            pdb_file: Some(CString::new("MapleStory.pdb").unwrap()),
            auto_login_data: Some(AutoLoginData {
                username: CString::new("admin").unwrap(),
                password: CString::new("test1234").unwrap(),
                world: 0,
                channel: 0,
                char_index: 0,
            })
        }
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();
