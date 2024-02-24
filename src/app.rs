use std::ffi::c_int;

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        DispatchMessageA, MsgWaitForMultipleObjects, PeekMessageA, TranslateMessage, MSG,
        PM_REMOVE, QUEUE_STATUS_FLAGS, WM_QUIT,
    },
};

use crate::{
    shroom_ffi::{
        self, socket::cclientsocket_manipulate_packet, CwvsAppRunRef,
        CCLIENT_SOCKET_SINGLETON, ISMSG,
    },
    static_lazy_hook,
};

static_lazy_hook!(_RUN_HOOK, CwvsAppRunRef, cwvs_app_run_hook);

unsafe extern "thiscall" fn cwvs_app_run_hook(
    this: *mut shroom_ffi::CWvsApp,
    terminate: *mut c_int,
) {
    if let Some(socket) = CCLIENT_SOCKET_SINGLETON.get_instance_mut() {
        cclientsocket_manipulate_packet()(socket as *mut _);
    }

    let app = this.as_mut().unwrap();
    loop {
        let dev_ix = MsgWaitForMultipleObjects(
            Some(&mut app.input_handles),
            false,
            0,
            QUEUE_STATUS_FLAGS(0xFF),
        );
        match dev_ix.0 {
            0..=2 => {
                let inp_sys = shroom_ffi::CINPUT_SYSTEM_SINGLETON
                    .get_instance_mut()
                    .unwrap();
                shroom_ffi::cinput_system_update_device()(inp_sys as *mut _, dev_ix.0 as c_int);

                let mut msg: ISMSG = std::mem::zeroed();
                while shroom_ffi::cinput_system_get_is_message()(inp_sys as *mut _, &mut msg) != 0 {
                    shroom_ffi::cwvs_app_is_msg_proc()(
                        app as *mut _,
                        msg.msg,
                        msg.wparam,
                        msg.lparam,
                    );
                }
            }
            3 => {
                let mut msg: MSG = std::mem::zeroed();
                while PeekMessageA(&mut msg as *mut MSG, HWND(0), 0, 0, PM_REMOVE) != false {
                    TranslateMessage(&msg);
                    DispatchMessageA(&msg);
                }

                if *terminate.as_ref().unwrap() != 0 || msg.message == WM_QUIT {
                    break;
                }
            }
            _ => unreachable!(),
        }
        //match dev_ix))
    }
}
