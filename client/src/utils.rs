use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use winapi::shared::minwindef::MAX_PATH;
use winapi::shared::minwindef::{LPARAM, WPARAM};
use winapi::shared::windef::RECT;
use winapi::um::shlobj::{SHGetFolderPathW, CSIDL_MYDOCUMENTS};
use winapi::um::winuser::*;

use cef_sys::cef_event_flags_t::*;

const CACHE_PATH: &str = "./GTA San Andreas User Files/CEF/";

pub fn is_key_pressed(key: i32) -> bool {
    let key_state = unsafe { GetKeyState(key) as u16 };
    key_state >> 15 == 1
}

pub fn cef_keyboard_modifiers(wparam: WPARAM, lparam: LPARAM) -> u32 {
    let mut modifiers = 0;

    if is_key_pressed(VK_SHIFT) {
        modifiers |= EVENTFLAG_SHIFT_DOWN;
    }

    if is_key_pressed(VK_CONTROL) {
        modifiers |= EVENTFLAG_CONTROL_DOWN;
    }

    if is_key_pressed(VK_MENU) {
        modifiers |= EVENTFLAG_ALT_DOWN;
    }

    // Low bit set from GetKeyState indicates "toggled".
    if unsafe { GetKeyState(VK_NUMLOCK) } & 1 == 1 {
        modifiers |= EVENTFLAG_NUM_LOCK_ON;
    }

    if unsafe { GetKeyState(VK_CAPITAL) } & 1 == 1 {
        modifiers |= EVENTFLAG_CAPS_LOCK_ON;
    }

    match wparam as i32 {
        VK_RETURN => {
            if (lparam >> 16) & KF_EXTENDED as isize != 0 {
                modifiers |= EVENTFLAG_IS_KEY_PAD;
            }
        }

        VK_INSERT | VK_DELETE | VK_HOME | VK_END | VK_PRIOR | VK_NEXT | VK_UP | VK_DOWN
        | VK_LEFT | VK_RIGHT => {
            if !(((lparam >> 16) & KF_EXTENDED as isize) != 0) {
                modifiers |= EVENTFLAG_IS_KEY_PAD;
            }
        }

        VK_NUMLOCK | VK_NUMPAD0 | VK_NUMPAD1 | VK_NUMPAD2 | VK_NUMPAD3 | VK_NUMPAD4
        | VK_NUMPAD5 | VK_NUMPAD6 | VK_NUMPAD7 | VK_NUMPAD8 | VK_NUMPAD9 | VK_DIVIDE
        | VK_MULTIPLY | VK_SUBTRACT | VK_ADD | VK_DECIMAL | VK_CLEAR => {
            modifiers |= EVENTFLAG_IS_KEY_PAD;
        }

        VK_SHIFT => {
            if is_key_pressed(VK_LSHIFT) {
                modifiers |= EVENTFLAG_IS_LEFT;
            } else if is_key_pressed(VK_RSHIFT) {
                modifiers |= EVENTFLAG_IS_RIGHT;
            }
        }

        VK_CONTROL => {
            if is_key_pressed(VK_LCONTROL) {
                modifiers |= EVENTFLAG_IS_LEFT;
            } else if is_key_pressed(VK_RCONTROL) {
                modifiers |= EVENTFLAG_IS_RIGHT;
            }
        }

        VK_MENU => {
            if is_key_pressed(VK_LMENU) {
                modifiers |= EVENTFLAG_IS_LEFT;
            } else if is_key_pressed(VK_RMENU) {
                modifiers |= EVENTFLAG_IS_RIGHT;
            }
        }

        VK_LWIN => {
            modifiers |= EVENTFLAG_IS_LEFT;
        }

        VK_RWIN => {
            modifiers |= EVENTFLAG_IS_RIGHT;
        }

        _ => (),
    }

    return modifiers as u32;
}

pub fn client_rect() -> [usize; 2] {
    let mut size = [0, 0];

    if let Some(hwnd) = client_api::wndproc::hwnd() {
        let mut rect = RECT {
            left: 0,
            right: 0,
            bottom: 0,
            top: 0,
        };

        unsafe {
            GetClientRect(hwnd, &mut rect);
            size = [
                (rect.right - rect.left) as usize,
                (rect.bottom - rect.top) as usize,
            ];
        }
    }

    size
}

#[inline(always)]
pub fn current_time() -> i128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as i128
}

pub fn documents_path() -> PathBuf {
    let mut buffer = vec![0; MAX_PATH];
    let mut path = PathBuf::new();

    let result = unsafe {
        SHGetFolderPathW(
            std::ptr::null_mut(),
            CSIDL_MYDOCUMENTS,
            std::ptr::null_mut(),
            0,
            buffer.as_mut_ptr(),
        )
    };

    if result == 0 {
        let null_idx = buffer.iter().position(|&ch| ch == 0).unwrap_or(0);
        let docs = OsString::from_wide(&buffer[0..null_idx]);

        path = path.join(docs).join(CACHE_PATH);
    }

    path
}

pub fn cef_dir() -> PathBuf {
    if let Some(path) = std::env::args()
        .skip_while(|arg| !arg.contains("--lp"))
        .skip(1)
        .next()
    {
        PathBuf::from(path).join("cef")
    } else {
        // в случае если игра запущена из другого места, а не с поомщью лаунчера
        let exe = std::env::current_exe().ok();

        exe.as_ref()
            .and_then(|exe| exe.parent())
            .map(|parent| parent.join("cef"))
            .unwrap_or_else(|| PathBuf::from("./cef"))
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RenderMode {
    DirectX,
    Renderware,
    Empty,
}

pub fn current_render_mode() -> RenderMode {
    let file = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|par| par.to_owned()))
        .map(|parent| parent.join("cef_render_directx.set"))
        .unwrap_or_else(|| PathBuf::from("./cef_render_directx.set"));

    if std::fs::metadata(file).is_ok() {
        RenderMode::DirectX
    } else {
        RenderMode::Renderware
    }
}
