use serde::{Deserialize, Serialize};
use std::{ffi::CString, fmt::Write, sync::OnceLock};

#[derive(Debug, Deserialize, Serialize)]
pub enum LogBackend {
    Console,
    File(String),
    Debug

}

#[derive(Debug, Deserialize, Serialize)]
pub struct AutoLoginData {
    pub username: CString,
    pub password: CString,
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
    pub pdb_file: Option<CString>,
    pub auto_login_data: Option<AutoLoginData>,
    pub window_data: Option<WindowData>,
    pub packet_tracing: Option<PacketTracingData>,
    pub multi_jump: Option<usize>,
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
            log_backend: LogBackend::Console,
            skip_logo: true,
            log_msgbox: false,
            pdb_file: Some(CString::new("MapleStory.pdb").unwrap()),
            auto_login_data: Some(AutoLoginData {
                username: CString::new("admin").unwrap(),
                password: CString::new("test1234").unwrap(),
                world: Some(0),
                channel: Some(0),
                char_index: Some(0),
            }),
            window_data: Some(WindowData {
                name: "DinputStory".to_string(),
                pid: true,
                time: true,
            }),
            packet_tracing: Some(PacketTracingData {
                send_file: "send_packets.txt".to_string(),
                recv_file: "recv_packets.txt".to_string(),
                log_data: false,
            }),
            multi_jump: Some(3)
        }
    }
}

pub static CONFIG: OnceLock<Config> = OnceLock::new();
