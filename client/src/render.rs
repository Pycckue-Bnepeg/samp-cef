use std::sync::{Arc, Mutex};
use winapi::shared::d3d9::IDirect3DDevice9;

use crate::browser::manager::Manager;

use client_api::gta::menu_manager::CMenuManager;

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
    client_api::gta::d9_proxy::set_proxy(on_create, on_render, on_reset);

    let render = Render { manager };

    unsafe {
        RENDER = Some(render);
    }
}

pub fn uninitialize() {
    unsafe {
        RENDER.take();
    }
}

fn on_create() {
    println!("device created!");
}

fn on_render(_: &mut IDirect3DDevice9) {
    let render = Render::get();

    {
        let mut manager = render.manager.lock().unwrap();
        manager.do_not_draw(CMenuManager::is_menu_active());
        manager.draw();
    }

    crate::app::mainloop();
}

fn on_reset(_: &mut IDirect3DDevice9, reset_flag: u8) {
    let render = Render::get();
    let mut manager = render.manager.lock().unwrap();

    match reset_flag {
        RESET_FLAG_PRE => manager.on_lost_device(),
        RESET_FLAG_POST => {
            manager.on_reset_device();
            let rect = crate::utils::client_rect();
            manager.resize(rect[0], rect[1]);
        }
        _ => (),
    }
}
