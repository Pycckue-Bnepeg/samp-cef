use std::sync::{Arc, Mutex};
use winapi::shared::d3d9::IDirect3DDevice9;

use crate::browser::manager::Manager;

const RESET_FLAG_PRE: u8 = 0;
const RESET_FLAG_POST: u8 = 1;

static mut RENDER: Option<Render> = None;

struct Render {
    manager: Arc<Mutex<Manager>>,
}

impl Render {
    fn get<'a>() -> &'a mut Render {
        unsafe {
            RENDER
                .as_mut()
                .expect("Unexpected null pointer to client::render::RENDER")
        }
    }
}

pub fn initialize(manager: Arc<Mutex<Manager>>) {
    while !client_api::gta::d3d9::set_proxy(Some(on_render), Some(on_reset)) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let render = Render { manager };

    unsafe {
        RENDER = Some(render);
    }
}

pub fn uninitialize() {
    client_api::gta::d3d9::unset_proxy();

    unsafe {
        RENDER.take();
    }
}

fn on_render(_: &mut IDirect3DDevice9) {
    let render = Render::get();
    let manager = render.manager.lock().unwrap();
    manager.draw();
}

fn on_reset(_: &mut IDirect3DDevice9, reset_flag: u8) {
    let render = Render::get();
    let manager = render.manager.lock().unwrap();

    match reset_flag {
        RESET_FLAG_PRE => manager.on_lost_device(),
        RESET_FLAG_POST => manager.on_reset_device(),
        _ => (),
    }
}
