use std::ffi::{c_int, c_uchar, c_uint, c_ushort, c_void};

use crate::fn_ref;

use super::{addr, ztl::{zxarr::ZArray, zxstr::ZXString8}};

#[derive(Debug)]
#[repr(C)]
pub struct COutPacket {
    pub is_loopback: c_int,
    pub send_buf: ZArray<c_uchar>,
    pub offset: c_uint,
    pub is_encrypted_by_shanda: c_int,
}
#[derive(Debug)]
#[repr(C)]
pub struct CInPacket {
    pub is_loopback: c_int,
    pub state: c_int,
    pub recv_buf: ZArray<c_uchar>,
    pub len: c_ushort,
    pub raw_seq: c_ushort,
    pub data_len: c_ushort,
    pub offset: c_uint,
}

fn_ref!(
    coutpacket_encode1,
    addr::COUTPACKET_ENCODE1,
    unsafe extern "thiscall" fn(*mut COutPacket, c_uchar)
);
fn_ref!(
    coutpacket_encode2,
    addr::COUTPACKET_ENCODE2,
    unsafe extern "thiscall" fn(*mut COutPacket, c_ushort)
);
fn_ref!(
    coutpacket_encode4,
    addr::COUTPACKET_ENCODE4,
    unsafe extern "thiscall" fn(*mut COutPacket, c_uint)
);
fn_ref!(
    coutpacket_encode_str,
    addr::COUTPACKET_ENCODE_STR,
    unsafe extern "thiscall" fn(*mut COutPacket, ZXString8)
);
fn_ref!(
    coutpacket_encode_buf,
    addr::COUTPACKET_ENCODE_BUF,
    unsafe extern "thiscall" fn(*mut COutPacket, *const c_void, c_uint)
);
fn_ref!(
    coutpacket_make_buffer_list,
    addr::COUTPACKET_MAKE_BUFFER_LIST,
    unsafe extern "thiscall" fn(*mut COutPacket, *const c_void, c_ushort, *mut c_uint, c_int, c_uint)
);

fn_ref!(
    cinpacket_decode1,
    addr::CINPACKET_DECODE1,
    unsafe extern "thiscall" fn(*mut CInPacket) -> c_uchar
);
fn_ref!(
    cinpacket_decode2,
    addr::CINPACKET_DECODE2,
    unsafe extern "thiscall" fn(*mut CInPacket) -> c_ushort
);
fn_ref!(
    cinpacket_decode4,
    addr::CINPACKET_DECODE4,
    unsafe extern "thiscall" fn(*mut CInPacket) -> c_uint
);
fn_ref!(
    cinpacket_decode_str,
    addr::CINPACKET_DECODE_STR,
    unsafe extern "thiscall" fn(*mut CInPacket, *mut ZXString8) -> ZXString8
);
fn_ref!(
    cinpacket_decode_buf,
    addr::CINPACKET_DECODE_BUF,
    unsafe extern "thiscall" fn(*mut CInPacket, *mut c_void, c_uint)
);


pub type CClientSocket = c_void;

fn_ref!(
    cclientsocket_send_packet,
    addr::CCLIENTSOCKET_SEND_PACKET,
    unsafe extern "thiscall" fn(*mut CClientSocket, *mut COutPacket)
);

fn_ref!(
    cclientsocket_process_packet,
    addr::CCLIENTSOCKET_PROCESS_PACKET,
    unsafe extern "thiscall" fn(*mut CClientSocket, *mut CInPacket)
);

fn_ref!(
    cclientsocket_manipulate_packet,
    0x4b0220,
    unsafe extern "thiscall" fn(*mut CClientSocket)
);

const SEND_PACKET_TRAMPOLINE_ENTRY: usize  = addr::CCLIENTSOCKET_SEND_PACKET + 5;

#[naked]
pub(crate) unsafe extern "fastcall" fn send_packet_trampoline(this: *mut CClientSocket, pkt: *mut COutPacket) {
    unsafe {
        std::arch::asm!(
            // Push packet param
            "push edx",
            // Push fake return address -> fake ret addy
            "push {1}",
            // Patched bytes for detour jump
            "push ebp",
            "mov ebp, esp",
            "push 0xffffffff",
            // Load address for jump
            "mov eax, {0}",
            "jmp eax",
            const SEND_PACKET_TRAMPOLINE_ENTRY,
            const addr::SOCKET_SINGLETON_SEND_PACKET_RET,
            options(noreturn)
        );
    }
}