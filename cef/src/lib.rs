#![feature(arbitrary_self_types)]
#![feature(core_intrinsics)]

use cef_sys::{cef_main_args_t, cef_settings_t};
use std::sync::Arc;

pub mod app;
pub mod browser;
pub mod client;
pub mod command_line;
pub mod handlers;
pub mod process_message;
pub mod ref_counted;
pub mod settings;
pub mod task;
pub mod types;
pub mod v8;

mod rust_to_c;

use crate::app::App;

pub fn execute_process<T: App>(args: &cef_main_args_t, app: Option<Arc<T>>) -> i32 {
    let app_ptr = app
        .map(|app| self::rust_to_c::app::wrap(app))
        .unwrap_or(std::ptr::null_mut());

    unsafe { cef_sys::cef_execute_process(args, app_ptr, std::ptr::null_mut()) }
}

pub fn initialize<T: App>(
    args: Option<&cef_main_args_t>, settings: &cef_settings_t, app: Option<Arc<T>>,
) -> i32 {
    let args = args
        .map(|args| args as *const _)
        .unwrap_or(std::ptr::null());

    let app_ptr = app
        .map(|app| self::rust_to_c::app::wrap(app))
        .unwrap_or(std::ptr::null_mut());

    unsafe { cef_sys::cef_initialize(args, settings, app_ptr, std::ptr::null_mut()) }
}

pub fn run_message_loop() {
    unsafe {
        cef_sys::cef_run_message_loop();
    }
}

pub fn do_message_loop_work() {
    unsafe {
        cef_sys::cef_do_message_loop_work();
    }
}

pub fn quit_message_loop() {
    unsafe {
        cef_sys::cef_quit_message_loop();
    }
}

pub fn shutdown() {
    unsafe {
        cef_sys::cef_shutdown();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ProcessId {
    None,
    Browser,
    Renderer,
}

impl Into<cef_sys::cef_process_id_t::Type> for ProcessId {
    fn into(self) -> cef_sys::cef_process_id_t::Type {
        match self {
            ProcessId::None => 0,
            ProcessId::Browser => 0,
            ProcessId::Renderer => 1,
        }
    }
}

impl From<cef_sys::cef_process_id_t::Type> for ProcessId {
    fn from(val: cef_sys::cef_process_id_t::Type) -> ProcessId {
        match val {
            0 => ProcessId::Browser,
            1 => ProcessId::Renderer,
            _ => ProcessId::None,
        }
    }
}
