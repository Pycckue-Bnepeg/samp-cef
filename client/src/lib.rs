#![allow(non_snake_case)]
#![feature(abi_thiscall)]
#![feature(arbitrary_self_types)]

use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

pub mod app;
pub mod browser;
pub mod network;
pub mod render;
pub mod utils;

#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(instance);
        }

        std::thread::spawn(|| {
            app::initialize();
        });
    }

    if reason == DLL_PROCESS_DETACH {
        app::uninitialize();
    }

    return true;
}
