use std::ffi::{c_int, c_void};

use windows::core::PCSTR;

use crate::fn_ref;

pub mod ztl;
pub mod socket;
pub mod error_codes;
/*pub mod com {
    pub mod iface;
}
pub mod client_socket;
*/

use self::ztl::zxstr::ZXString8;

pub type CLogo = c_void;
pub type CLogin = c_void;
pub type CUIAvatar = c_void;
pub type CWvsApp = c_void;

pub mod addr {
    pub const CLOGO_INIT: usize = 0x60e240;
    pub const CLOGO_END: usize = 0x60bd00;
    pub const CMSGBOX_INIT: usize = 0x669370;

    pub const CLOGIN_INIT: usize = 0x5d8010;
    pub const CLOGIN_SEND_CHECK_PASSWORD_PACKET: usize = 0x5db9d0;
    pub const CLOGIN_SEND_LOGIN_PACKET: usize = 0x5dbef0;
    pub const CLOGIN_SEND_SELECT_CHAR_PACKET: usize = 0x5da2a0;
    pub const CLOGIN_ON_RECOMMEND_WORLD_MESSAGE: usize = 0x5d7280;

    pub const CUIAVATAR_SELECT_CHARACTER: usize = 0x5ea280;

    pub const CWVS_APP_INITIALIZE_GAME_DATA: usize = 0x9c8440;

    pub const CCLIENTSOCKET_SEND_PACKET: usize = 0x004af9f0;
    pub const CCLIENTSOCKET_PROCESS_PACKET: usize = 0x004b00f0;
    // Only required if the Send Packet function checks the return address(+95?)
    pub const SOCKET_SINGLETON_SEND_PACKET_RET: usize = 0x00429b8b + 5;
    pub const SEND_PACKET_RET_SPOOF: bool = true;

    pub const COUTPACKET_ENCODE1: usize = 0x00415360;
    pub const COUTPACKET_ENCODE2: usize = 0x0042ca10;
    pub const COUTPACKET_ENCODE4: usize = 0x004153b0;
    pub const COUTPACKET_ENCODE_STR: usize = 0x004841f0;
    pub const COUTPACKET_ENCODE_BUF: usize = 0x00482200;

    pub const CINPACKET_DECODE1: usize = 0x4097d0;
    pub const CINPACKET_DECODE2: usize = 0x42a2a0;
    pub const CINPACKET_DECODE4: usize = 0x409870;
    pub const CINPACKET_DECODE_STR: usize = 0x484140;
    pub const CINPACKET_DECODE_BUF: usize = 0x4336a0;
}

pub mod addr92 {
    pub const CLOGO_INIT: usize = 0x602730;
    pub const CLOGO_END: usize = 0x600da0;
    pub const CMSGBOX_INIT: usize = 0x65c7b0;

    pub const CLOGIN_INIT: usize = 0x5ce780;
    pub const CLOGIN_SEND_CHECK_PASSWORD_PACKET: usize = 0x5d2190;
    pub const CLOGIN_SEND_LOGIN_PACKET: usize = 0x5d26b0;
    pub const CLOGIN_SEND_SELECT_CHAR_PACKET: usize = 0x5d0a60;
    pub const CLOGIN_ON_RECOMMEND_WORLD_MESSAGE: usize = 0x5cd030;

    pub const CUIAVATAR_SELECT_CHARACTER: usize = 0x5e0880;

    pub const CWVS_APP_INITIALIZE_GAME_DATA: usize = 0x99dc00;

    pub const CCLIENTSOCKET_SEND_PACKET: usize = 0x004af9f0;
    pub const CCLIENTSOCKET_PROCESS_PACKET: usize = 0x004b00f0;
    // Only required if the Send Packet function checks the return address(+95?)
    pub const SOCKET_SINGLETON_SEND_PACKET_RET: usize = 0;
    pub const SEND_PACKET_RET_SPOOF: bool = false;

    pub const COUTPACKET_ENCODE1: usize = 0x415b70;
    pub const COUTPACKET_ENCODE2: usize = 0x42d3b0;
    pub const COUTPACKET_ENCODE4: usize = 0x415bc0;
    pub const COUTPACKET_ENCODE_STR: usize = 0x480c10;
    pub const COUTPACKET_ENCODE_BUF: usize = 0x47eb20;

    pub const CINPACKET_DECODE1: usize = 0x409c00;
    pub const CINPACKET_DECODE2: usize = 0x42acd0;
    pub const CINPACKET_DECODE4: usize = 0x409ca0;
    pub const CINPACKET_DECODE_STR: usize = 0x480b60;
    pub const CINPACKET_DECODE_BUF: usize = 0x4347a0;
}

fn_ref!(
    cwvs_app_initialize_game_data,
    addr::CWVS_APP_INITIALIZE_GAME_DATA,
    unsafe extern "thiscall" fn(*mut CWvsApp)
);

fn_ref!(
    clogo_init,
    addr::CLOGO_INIT,
    unsafe extern "thiscall" fn(*mut CLogo, param: *const c_void)
);

fn_ref!(
    clogo_end,
    addr::CLOGO_END,
    unsafe extern "thiscall" fn(*mut CLogo)
);

fn_ref!(
    cmsgbox_init,
    addr::CMSGBOX_INIT,
    unsafe extern "thiscall" fn(*mut c_void, msg: ZXString8, link: ZXString8, desc: ZXString8)
);

fn_ref!(
    clogin_init,
    addr::CLOGIN_INIT,
    unsafe extern "thiscall" fn(*const CLogin, *const c_void)
);

fn_ref!(
    clogin_send_check_password_packet,
    addr::CLOGIN_SEND_CHECK_PASSWORD_PACKET,
    unsafe extern "thiscall" fn(*const CLogin, PCSTR, PCSTR)
);

fn_ref!(
    clogin_send_login_packet,
    addr::CLOGIN_SEND_LOGIN_PACKET,
    unsafe extern "thiscall" fn(*const CLogin, c_int, c_int)
);

fn_ref!(
    clogin_send_select_character_packet,
    addr::CLOGIN_SEND_SELECT_CHAR_PACKET,
    unsafe extern "thiscall" fn(*const CLogin)
);

fn_ref!(
    clogin_on_recommend_world_message,
    addr::CLOGIN_ON_RECOMMEND_WORLD_MESSAGE,
    unsafe extern "thiscall" fn(*const CLogin, *const c_void)
);

fn_ref!(
    cuiavatar_select_character,
    addr::CUIAVATAR_SELECT_CHARACTER,
    unsafe extern "thiscall" fn(*const CUIAvatar, c_int)
);
