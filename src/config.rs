use serde::{Deserialize, Serialize};
use widestring::U16CString;
use windows::core::{PCSTR, PCWSTR};
use std::{ffi::CString, fmt::Write, sync::OnceLock};

#[derive(Debug)]
pub struct Str(pub CString);

impl Str {
    pub fn new(s: &str) -> Self {
        Self(CString::new(s).expect("cstr"))
    }   

    pub fn as_pcstr(&self) -> PCSTR {
        PCSTR(self.0.as_ptr() as *const u8)
    }
}

impl<'de> Deserialize<'de> for Str {
    fn deserialize<D>(deserializer: D) -> Result<Str, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Str(CString::new(s).map_err(serde::de::Error::custom)?))
    }
}

impl Serialize for Str {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_str().map_err(serde::ser::Error::custom)?.serialize(serializer)
    }
}

#[derive(Debug)]
pub struct WString(pub U16CString);

impl WString {
    pub fn new(s: &str) -> Self {
        Self(U16CString::from_str(s).expect("cstr"))
    }   

    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.0.as_ptr() as *const u16)
    }
}

impl<'de> Deserialize<'de> for WString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self(U16CString::from_str(s).map_err(serde::de::Error::custom)?))
    }
}

impl Serialize for WString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.to_string().map_err(serde::ser::Error::custom)?.serialize(serializer)
    }
}


#[derive(Debug, Deserialize, Serialize)]
pub enum LogBackend {
    Console,
    File(String),
    Debug,
    Stdout
}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutoLoginData {
    pub username: Str,
    pub password: Str,
    pub world: Option<u32>,
    pub channel: Option<u32>,
    pub char_index: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PacketTracingData {
    pub send_file: String,
    pub recv_file: String,
    pub log_data: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WindowData {
    pub name: String,
    pub pid: bool,
    pub time: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WzFileData {
    pub version: WString,
    pub path: Option<String>
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WzImageData {
    pub path: String,
    pub retain_delay: usize,
    pub cachte_delay: usize
}

#[derive(Debug, Deserialize, Serialize)]
pub enum WzData {
    Image(WzImageData),
    Wz(WzFileData),
    Default
}

impl WzData {
    pub fn is_default(&self) -> bool {
        matches!(self, WzData::Default)
    }


    pub fn is_image(&self) -> bool {
        matches!(self, WzData::Image(_))
    }

    pub fn as_image(&self) -> Option<&WzImageData> {
        match self {
            WzData::Image(image) => Some(image),
            _ => None
        }
    }

    pub fn is_wz(&self) -> bool {
        matches!(self, WzData::Wz(_))
    }


    pub fn as_wz(&self) -> Option<&WzFileData> {
        match self {
            WzData::Wz(wz) => Some(wz),
            _ => None
        }
    }
}

impl AutoLoginData {
    pub fn get_world_channel(&self) -> Option<(u32, u32)> {
        self.world.zip(self.channel)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub log_backend: LogBackend,
    pub skip_logo: bool,
    pub log_msgbox: bool,
    pub pdb_file: Option<Str>,
    pub auto_login_data: Option<AutoLoginData>,
    pub window_data: Option<WindowData>,
    pub packet_tracing: Option<PacketTracingData>,
    pub multi_jump: Option<usize>,
    pub extra_dlls: Vec<Str>,
    pub disable_shanda: bool,
    pub handle_exceptions: bool,
    pub wz: WzData,
    pub lazy_tmpl_loading: bool,
}

impl Config {
    pub fn window_title(&self) -> Option<CString> {
        let wnd = self.window_data.as_ref()?;

        let mut name = String::with_capacity(64);
        name.push_str(&wnd.name);
        if wnd.pid {
            write!(&mut name, " - PID: {}", std::process::id()).expect("window pid");
        }
        if wnd.time {
            write!(&mut name, " - Time: {}", chrono::Local::now().format("%c")).expect("window time");
        }
        Some(CString::new(name).expect("cstr"))
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            log_backend: LogBackend::Stdout,
            skip_logo: true,
            log_msgbox: false,
            pdb_file: Some(Str::new("MapleStory.pdb")),
            auto_login_data: Some(AutoLoginData {
                username: Str::new("admin"),
                password: Str::new("test1234"),
                world: Some(0),
                channel: Some(0),
                char_index: Some(0),
            }),
            window_data: Some(WindowData {
                name: "DinputStory".to_string(),
                pid: true,
                time: true,
            }),
           /*  packet_tracing: Some(PacketTracingData {
                send_file: "send_packets.txt".to_string(),
                recv_file: "recv_packets.txt".to_string(),
                log_data: false,
            }),*/
            packet_tracing: None,
            multi_jump: Some(2),
            extra_dlls: Vec::default(),
            disable_shanda: true,
            handle_exceptions: true,
            wz: WzData::Wz(WzFileData {
                version: WString::new("95"),
                path: Some("wz95".to_string())
            }),
            lazy_tmpl_loading: true
            /*wz: WzData::Image(
                WzImageData {
                    path: "Data".to_string(),
                    retain_delay: 1000,
                    cachte_delay: 1000
                }
            )*/
        }
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();


#[cfg(test)]
mod tests {
    use super::*;

    fn gen_default_config() {
        let toml = toml::to_string_pretty(&Config::default()).unwrap();
        std::fs::write("config.toml", toml).unwrap();
    }

    #[test]
    fn config() {
        gen_default_config();
        toml::from_str::<Config>(&std::fs::read_to_string("config.toml").unwrap()).unwrap();        
    }
}
