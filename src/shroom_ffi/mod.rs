use std::ffi::{c_char, c_int, c_uchar, c_uint, c_void};

use windows::{
    core::{HRESULT, PCSTR, PCWSTR},
    Win32::{
        Foundation::{HANDLE, HWND},
        UI::WindowsAndMessaging::HHOOK,
    },
};

use crate::fn_ref;

pub mod error_codes;
pub mod socket;
pub mod ztl;
/*pub mod com {
    pub mod iface;
}
pub mod client_socket;
*/

use self::{
    socket::CClientSocket,
    ztl::{zxarr::ZArray, zxstr::ZXString8, TSingleton},
};

pub type CLogo = c_void;
pub type CLogin = c_void;
pub type CUIAvatar = c_void;
pub type CStaticFoothold = c_void;
pub type CInputSystem = c_void;
pub type IWzPackage = c_void;
pub type IWzSeekableArchive = c_void;
pub type IWzFileSystem = c_void;

#[repr(C)]
pub struct BStrData {
    pub wstr: PCWSTR,
    pub str: PCSTR,
    pub ref_count: c_int,
}

#[repr(C)]
pub struct ZtlBstrT(pub *mut BStrData);

impl ZtlBstrT {
    pub fn as_wstr(&self) -> Option<PCWSTR> {
        if self.0.is_null() {
            None
        } else {
            unsafe { Some((*self.0).wstr) }
        }
    }
}

#[repr(transparent)]
pub struct Padding<const N: usize>([u8; N]);

impl<const N: usize> std::fmt::Debug for Padding<N> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Padding").field("len", &N).finish()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct TagPoint {
    pub x: c_int,
    pub y: c_int,
}

#[derive(Debug)]
#[repr(C)]
pub struct CUserLocal {
    pub padding: Padding<0x467c>,
    pub last_jump: c_int,
    pub pt_before_key_down: TagPoint,
    pub last_key_down: c_int,
    pub wings_end: c_int,
    pub key_down: c_int,
    pub key_down_scan_code: c_uint,
}

#[derive(Debug)]
#[repr(C)]
pub struct CVecCtrl {
    pub padding: Padding<0x1a0>,
    pub foothold: *mut CStaticFoothold,
}

impl CVecCtrl {
    pub fn is_on_foothold(&self) -> bool {
        !self.foothold.is_null()
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct CWvsApp {
    pub vtable: *const c_void,
    pub hwnd: HWND,
    pub pcominitialized: c_int,
    pub main_thread_id: c_uint,
    pub hook: *const HHOOK,
    pub is_win9x: c_int,
    pub os_version: c_int,
    pub os_minor_version: c_int,
    pub osbuild_number: c_int,
    pub csdversion: ZXString8,
    pub has_64_bit_info: c_int,
    pub update_time: c_int,
    pub first_update: c_int,
    pub cmd_line: ZXString8,
    pub game_start_mode: c_int,
    pub auto_connect: c_int,
    pub show_ad_balloon: c_int,
    pub exit_by_title_escape: c_int,
    pub zexception_code: HRESULT,
    pub com_error_code: HRESULT,
    pub security_error_code: c_uint,
    pub target_version: c_int,
    pub last_server_ipcheck: c_int,
    pub last_server_ipcheck2: c_int,
    pub last_gghooking_apicheck: c_int,
    pub last_security_check: c_int,
    pub input_handles: [HANDLE; 3],
    pub next_security_check: c_int,
    pub is_enabled_dx9: c_uchar,
    pub backup_buffer: ZArray<c_uchar>,
    pub backup_buffer_size: c_uint,
    pub clear_stack_log: c_uint,
    pub window_active: c_int,
}

static_assertions::assert_eq_size!(CWvsApp, [u8; 0x8c]);

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
    pub const COUTPACKET_MAKE_BUFFER_LIST: usize = 0x68d100;

    pub const CINPACKET_DECODE1: usize = 0x4097d0;
    pub const CINPACKET_DECODE2: usize = 0x42a2a0;
    pub const CINPACKET_DECODE4: usize = 0x409870;
    pub const CINPACKET_DECODE_STR: usize = 0x484140;
    pub const CINPACKET_DECODE_BUF: usize = 0x4336a0;

    pub const CIOBUFFER_MANIPULATOR_EN: usize = 0x68c8e0;
    pub const CIOBUFFER_MANIPULATOR_DE: usize = 0x68cab0;

    pub const CUSERLOCAL_JUMP: usize = 0x90a1d0;
    pub const CUSERLOCAL_IS_IMMOVABLE: usize = 0x905430;

    pub const CVEC_CTRL_JUST_JUMP: usize = 0x993ea0;
    pub const CVEC_CTRL_IS_SWIMMING: usize = 0x6a0160;

    pub const GET_UPDATE_TIME: usize = 0x95b290;
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

    pub const USE_SEND_PACKET_TRAMPOLINE: bool = false;
}

fn_ref!(
    get_update_time,
    addr::GET_UPDATE_TIME,
    unsafe extern "cdecl" fn() -> c_int
);

fn_ref!(
    cuserlocal_jump,
    addr::CUSERLOCAL_JUMP,
    unsafe extern "thiscall" fn(*mut CUserLocal, c_int)
);

fn_ref!(
    cuserlocal_is_immovable,
    addr::CUSERLOCAL_IS_IMMOVABLE,
    unsafe extern "thiscall" fn(*mut CUserLocal) -> c_int
);

fn_ref!(
    cvec_ctrl_just_jump,
    addr::CVEC_CTRL_JUST_JUMP,
    unsafe extern "thiscall" fn(*mut CVecCtrl) -> c_int
);

fn_ref!(
    cvec_ctrl_is_swimming,
    addr::CVEC_CTRL_IS_SWIMMING,
    unsafe extern "thiscall" fn(*mut CVecCtrl) -> c_int
);

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

fn_ref!(
    ciobuffer_manipulator_en,
    addr::CIOBUFFER_MANIPULATOR_EN,
    unsafe extern "stdcall" fn(*mut c_char, c_int) -> c_uchar
);

fn_ref!(
    ciobuffer_manipulator_de,
    addr::CIOBUFFER_MANIPULATOR_DE,
    unsafe extern "stdcall" fn(*mut c_char, c_int) -> c_uchar
);

pub static mut CCLIENT_SOCKET_SINGLETON: TSingleton<CClientSocket> =
    unsafe { std::mem::transmute(0xc64064) };

fn_ref!(
    cwvs_app_run,
    0x9c5f00,
    unsafe extern "thiscall" fn(*mut CWvsApp, *mut c_int)
);

fn_ref!(
    cwvs_app_is_msg_proc,
    0x9c1ce0,
    unsafe extern "thiscall" fn(*mut CWvsApp, c_uint, c_uint, c_int)
);

fn_ref!(
    cinput_system_update_device,
    0x571710,
    unsafe extern "thiscall" fn(this: *mut CInputSystem, dev_ix: c_int)
);

pub static mut CINPUT_SYSTEM_SINGLETON: TSingleton<CInputSystem> =
    unsafe { std::mem::transmute(0xc68c20) };

#[derive(Debug)]
#[repr(C)]
pub struct ISMSG {
    pub msg: c_uint,
    pub wparam: c_uint,
    pub lparam: c_int,
}

fn_ref!(
    cinput_system_get_is_message,
    0x5708f0,
    unsafe extern "thiscall" fn(*mut CInputSystem, msg: *mut ISMSG) -> c_int
);

fn_ref!(
    cinput_system_generate_auto_key_down,
    0x56f990,
    unsafe extern "thiscall" fn(*mut CInputSystem, msg: *mut ISMSG) -> c_int
);

fn_ref!(
    iwz_package_init,
    0x9c8ec0,
    unsafe extern "thiscall" fn(
        *mut IWzPackage,
        key: ZtlBstrT,
        base_uol: ZtlBstrT,
        archive: *mut IWzSeekableArchive,
    ) -> HRESULT
);

fn_ref!(
    iwz_filesystem_init,
    0x9c8e40,
    unsafe extern "thiscall" fn(
        *mut IWzFileSystem,
        path: ZtlBstrT,
    ) -> HRESULT
);


fn_ref!(
    bstr_assign,
    0x416e50,
    unsafe extern "thiscall" fn(*mut ZtlBstrT, PCWSTR) -> *mut BStrData
);

fn_ref!(
    bstr_ctor,
    0x4032f0,
    unsafe extern "thiscall" fn(*mut ZtlBstrT, PCWSTR) -> *mut BStrData
);