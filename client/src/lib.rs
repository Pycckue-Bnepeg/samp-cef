#![allow(non_snake_case)]
#![feature(abi_thiscall)]
#![feature(arbitrary_self_types)]

use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

use simplelog::{CombinedLogger, LevelFilter, WriteLogger};
use std::fs::File;

pub mod app;
pub mod browser;

#[cfg(feature = "crash_logger")]
pub mod crash_logger;
pub mod external;
pub mod network;
pub mod render;
pub mod rodio_audio;
pub mod utils;

// TODO: Сделать человеческие модули звука

#[cfg(not(feature = "rodio_audio"))]
pub mod audio;

#[cfg(feature = "rodio_audio")]
pub mod audio {
    pub use crate::rodio_audio::*;
}

#[no_mangle]
pub extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(instance);
        }

        let mut config = simplelog::ConfigBuilder::new();

        let config = config
            .add_filter_allow_str("client")
            .add_filter_allow_str("client_api")
            .set_max_level(LevelFilter::Trace)
            .build();

        CombinedLogger::init(vec![WriteLogger::new(
            LevelFilter::Trace,
            config,
            File::create("cef_client.log").unwrap(),
        )])
        .unwrap();

        std::thread::spawn(|| {
            #[cfg(feature = "crash_logger")]
            crash_logger::initialize();

            app::initialize();
        });
    }

    if reason == DLL_PROCESS_DETACH {
        log::trace!("DllMain reason == DLL_PROCESS_DETACH calling unitialize");
        app::uninitialize();
    }

    true
}
