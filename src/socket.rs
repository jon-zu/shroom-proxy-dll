use std::{
    ffi::{c_uchar, c_uint, c_ushort, c_void},
    sync::{LazyLock, Mutex},
};

use crate::{
    config::{PacketTracingData, CONFIG},
    hook_list, lazy_hook, ret_addr,
    shroom_ffi::{
        addr,
        socket::{
            cclientsocket_process_packet, cclientsocket_send_packet, cinpacket_decode1,
            cinpacket_decode2, cinpacket_decode4, cinpacket_decode_buf, cinpacket_decode_str,
            coutpacket_encode1, coutpacket_encode2, coutpacket_encode4, coutpacket_encode_buf,
            coutpacket_encode_str, send_packet_trampoline, CClientSocket, CInPacket, COutPacket,
            CclientsocketProcessPacket, CclientsocketSendPacket, CinpacketDecode1,
            CinpacketDecode2, CinpacketDecode4, CinpacketDecodeBuf, CinpacketDecodeStr,
            CoutpacketEncode1, CoutpacketEncode2, CoutpacketEncode4, CoutpacketEncodeBuf,
            CoutpacketEncodeStr,
        },
        ztl::zxstr::ZXString8,
    },
    util::{
        hooks::LazyHook,
        packet_schema::{PacketStructElem, PacketStructLogger, ShroomPacket},
    },
};

pub fn tracing_data() -> &'static PacketTracingData {
    CONFIG.get().unwrap().packet_tracing.as_ref().unwrap()
}

static SEND_CTX: LazyLock<Mutex<PacketStructLogger<COutPacket>>> = LazyLock::new(|| {
    Mutex::new(PacketStructLogger::new(
        tracing_data().send_file.clone(),
        tracing_data().log_data,
    ))
});

static RECV_CTX: LazyLock<Mutex<PacketStructLogger<CInPacket>>> = LazyLock::new(|| {
    Mutex::new(PacketStructLogger::new(
        tracing_data().recv_file.clone(),
        tracing_data().log_data,
    ))
});

macro_rules! add_send_elem {
    ($pkt:ident, $v:ident) => {
        let ret_addr = ret_addr!();
        let pkt = $pkt.as_ref().unwrap();
        SEND_CTX
            .lock()
            .unwrap()
            .add_elem(PacketStructElem::new(pkt.offset(), ret_addr, $v));
    };
}

macro_rules! add_recv_elem {
    ($pkt:ident, $v:ident) => {
        let ret_addr = ret_addr!();
        let pkt = $pkt.as_ref().unwrap();
        RECV_CTX
            .lock()
            .unwrap()
            .add_elem(PacketStructElem::new(pkt.offset(), ret_addr, $v));
    };
}

static COUTPACKET_ENCODE1_HOOK: LazyHook<CoutpacketEncode1> =
    lazy_hook!(coutpacket_encode1, coutpacket_encode1_hook);
unsafe extern "thiscall" fn coutpacket_encode1_hook(this: *mut COutPacket, v: c_uchar) {
    add_send_elem!(this, v);
    COUTPACKET_ENCODE1_HOOK.call(this, v)
}

static COUTPACKET_ENCODE2_HOOK: LazyHook<CoutpacketEncode2> =
    lazy_hook!(coutpacket_encode2, coutpacket_encode2_hook);
unsafe extern "thiscall" fn coutpacket_encode2_hook(this: *mut COutPacket, v: c_ushort) {
    add_send_elem!(this, v);
    COUTPACKET_ENCODE2_HOOK.call(this, v)
}

static COUTPACKET_ENCODE4_HOOK: LazyHook<CoutpacketEncode4> =
    lazy_hook!(coutpacket_encode4, coutpacket_encode4_hook);
unsafe extern "thiscall" fn coutpacket_encode4_hook(this: *mut COutPacket, v: c_uint) {
    add_send_elem!(this, v);
    COUTPACKET_ENCODE4_HOOK.call(this, v)
}

static COUTPACKET_ENCODE_STR_HOOK: LazyHook<CoutpacketEncodeStr> =
    lazy_hook!(coutpacket_encode_str, coutpacket_encode_str_hook);
unsafe extern "thiscall" fn coutpacket_encode_str_hook(this: *mut COutPacket, v: ZXString8) {
    let v_ref = &v;
    add_send_elem!(this, v_ref);
    COUTPACKET_ENCODE_STR_HOOK.call(this, v)
}

static COUTPACKET_ENCODE_BUF_HOOK: LazyHook<CoutpacketEncodeBuf> =
    lazy_hook!(coutpacket_encode_buf, coutpacket_encode_buf_hook);
unsafe extern "thiscall" fn coutpacket_encode_buf_hook(
    this: *mut COutPacket,
    p: *const c_void,
    len: c_uint,
) {
    let slice = std::slice::from_raw_parts(p as *const u8, len as usize);
    add_send_elem!(this, slice);
    COUTPACKET_ENCODE_BUF_HOOK.call(this, p, len)
}

static CCLIENTSOCKET_SEND_PACKET_HOOK: LazyHook<CclientsocketSendPacket> =
    lazy_hook!(cclientsocket_send_packet, cclientsocket_send_packet_hook);

unsafe extern "thiscall" fn cclientsocket_send_packet_hook(
    this: *mut CClientSocket,
    pkt: *mut COutPacket,
) {
    let ret = ret_addr!();
    SEND_CTX
        .lock()
        .expect("send")
        .finish_send(ret, pkt.as_ref().unwrap());

    if addr::SEND_PACKET_RET_SPOOF {
        send_packet_trampoline(this, pkt);
    } else {
        CCLIENTSOCKET_SEND_PACKET_HOOK.call(this, pkt);
    }
}

static CINPACKET_DECODE1_HOOK: LazyHook<CinpacketDecode1> =
    lazy_hook!(cinpacket_decode1, cinpacket_decode1_hook);
unsafe extern "thiscall" fn cinpacket_decode1_hook(this: *mut CInPacket) -> c_uchar {
    let v = CINPACKET_DECODE1_HOOK.call(this);
    add_recv_elem!(this, v);
    v
}

static CINPACKET_DECODE2_HOOK: LazyHook<CinpacketDecode2> =
    lazy_hook!(cinpacket_decode2, cinpacket_decode2_hook);
unsafe extern "thiscall" fn cinpacket_decode2_hook(this: *mut CInPacket) -> c_ushort {
    let v = CINPACKET_DECODE2_HOOK.call(this);
    add_recv_elem!(this, v);
    v
}

static CINPACKET_DECODE4_HOOK: LazyHook<CinpacketDecode4> =
    lazy_hook!(cinpacket_decode4, cinpacket_decode4_hook);
unsafe extern "thiscall" fn cinpacket_decode4_hook(this: *mut CInPacket) -> c_uint {
    let v = CINPACKET_DECODE4_HOOK.call(this);
    add_recv_elem!(this, v);
    v
}

static CINPACKET_DECODE_STR_HOOK: LazyHook<CinpacketDecodeStr> =
    lazy_hook!(cinpacket_decode_str, cinpacket_decode_str_hook);
unsafe extern "thiscall" fn cinpacket_decode_str_hook(
    this: *mut CInPacket,
    out: *mut ZXString8,
) -> ZXString8 {
    let v = CINPACKET_DECODE_STR_HOOK.call(this, out);
    let v_ref = &v;
    add_recv_elem!(this, v_ref);
    v
}

static CINPACKET_DECODE_BUF_HOOK: LazyHook<CinpacketDecodeBuf> =
    lazy_hook!(cinpacket_decode_buf, cinpacket_decode_buf_hook);
unsafe extern "thiscall" fn cinpacket_decode_buf_hook(
    this: *mut CInPacket,
    p: *mut c_void,
    len: c_uint,
) {
    CINPACKET_DECODE_BUF_HOOK.call(this, p, len);
    let slice = std::slice::from_raw_parts(p as *const u8, len as usize);
    add_recv_elem!(this, slice);
}

static CCLIENTSOCKET_PROCESS_PACKET_HOOK: LazyHook<CclientsocketProcessPacket> = lazy_hook!(
    cclientsocket_process_packet,
    cclientsocket_process_packet_hook
);

unsafe extern "thiscall" fn cclientsocket_process_packet_hook(
    this: *mut CClientSocket,
    pkt: *mut CInPacket,
) {
    RECV_CTX
        .lock()
        .expect("recv")
        .set_packet_data(pkt.as_ref().unwrap());
    CCLIENTSOCKET_PROCESS_PACKET_HOOK.call(this, pkt);
    RECV_CTX
        .lock()
        .expect("recv")
        .finish_process(pkt.as_ref().unwrap());
}

hook_list!(
    PacketHooks,
    CINPACKET_DECODE1_HOOK,
    CINPACKET_DECODE2_HOOK,
    CINPACKET_DECODE4_HOOK,
    CINPACKET_DECODE_STR_HOOK,
    CINPACKET_DECODE_BUF_HOOK,
    CCLIENTSOCKET_PROCESS_PACKET_HOOK,
    COUTPACKET_ENCODE1_HOOK,
    COUTPACKET_ENCODE2_HOOK,
    COUTPACKET_ENCODE4_HOOK,
    COUTPACKET_ENCODE_STR_HOOK,
    COUTPACKET_ENCODE_BUF_HOOK,
    CCLIENTSOCKET_SEND_PACKET_HOOK,
);
