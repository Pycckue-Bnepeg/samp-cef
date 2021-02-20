#![allow(non_snake_case)]
#![feature(abi_thiscall)]
#![feature(arbitrary_self_types)]

use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use simplelog::{CombinedLogger, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::File;

pub mod app;
pub mod audio;
pub mod browser;
pub mod external;
pub mod network;
pub mod render;
pub mod utils;

#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(instance);
        }

        render::preinitialize();

        std::thread::spawn(|| {
            CombinedLogger::init(vec![
                TermLogger::new(LevelFilter::Trace, Config::default(), TerminalMode::Mixed),
                WriteLogger::new(
                    LevelFilter::Trace,
                    Config::default(),
                    File::create("cef_client.log").unwrap(),
                ),
            ])
            .unwrap();

            app::initialize();
        });
    }

    if reason == DLL_PROCESS_DETACH {
        app::uninitialize();
    }

    return true;
}
