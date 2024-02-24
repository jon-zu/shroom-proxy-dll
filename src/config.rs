use serde::{Deserialize, Serialize};
use windows::core::PCSTR;
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


#[derive(Debug, Deserialize, Serialize)]
pub enum LogBackend {
    Console,
    File(String),
    Debug

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
    pub handle_exceptions: bool
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
            log_backend: LogBackend::Console, //LogBackend::File("shroom.log".to_string()),
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
            handle_exceptions: true
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
