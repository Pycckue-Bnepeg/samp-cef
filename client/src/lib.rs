#![allow(non_snake_case)]

use std::fs::File;
use std::sync::Once;

use simplelog::{CombinedLogger, LevelFilter, WriteLogger};
use winapi::shared::minwindef::HMODULE;
use winapi::um::libloaderapi::DisableThreadLibraryCalls;
use winapi::um::winnt::DLL_PROCESS_ATTACH;

pub mod app;
pub mod browser;

#[cfg(feature = "crash_logger")]
pub mod crash_logger;
pub mod external;
pub mod network;
pub mod render;
pub mod rodio_audio;
pub mod static_cell;
pub mod utils;

// TODO: Сделать человеческие модули звука

#[cfg(not(feature = "rodio_audio"))]
pub mod audio;

#[cfg(feature = "rodio_audio")]
pub mod audio {
    pub use crate::rodio_audio::*;
}

static INIT: Once = Once::new();

fn initialize_logging() {
    let Ok(log_file) = File::create("cef_client.log") else {
        return;
    };

    let config = simplelog::ConfigBuilder::new()
        .add_filter_allow_str("client")
        .add_filter_allow_str("client_api")
        .set_max_level(LevelFilter::Trace)
        .build();

    let _ = CombinedLogger::init(vec![WriteLogger::new(
        LevelFilter::Trace,
        config,
        log_file,
    )]);
}

#[unsafe(no_mangle)]
pub extern "C" fn cef_client_initialize() {
    INIT.call_once(|| {
        initialize_logging();

        #[cfg(feature = "crash_logger")]
        crash_logger::initialize();

        app::initialize();
    });
}

/// # Safety
/// `instance` must be a valid module handle provided by the loader.
#[unsafe(no_mangle)]
pub unsafe extern "stdcall" fn DllMain(instance: HMODULE, reason: u32, _reserved: u32) -> bool {
    if reason == DLL_PROCESS_ATTACH {
        unsafe {
            DisableThreadLibraryCalls(instance);
        }
    }

    true
}
