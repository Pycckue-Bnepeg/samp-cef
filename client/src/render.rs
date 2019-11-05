use std::sync::{Arc, Mutex};
use winapi::shared::d3d9::IDirect3DDevice9;

use crate::browser::manager::Manager;

use client_api::gta::entity::CEntity;
use client_api::gta::menu_manager::CMenuManager;

use detour::GenericDetour;

const RESET_FLAG_PRE: u8 = 0;
const RESET_FLAG_POST: u8 = 1;

static mut RENDER: Option<Render> = None;

struct Render {
    manager: Arc<Mutex<Manager>>,
    centity_render: GenericDetour<extern "thiscall" fn(obj: *mut CEntity)>,
    init: bool,
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

    let centity_render = unsafe {
        let render_func: extern "thiscall" fn(*mut CEntity) = std::mem::transmute(0x00534310);
        let centity_render = GenericDetour::new(render_func, centity_render).unwrap();
        centity_render.enable().unwrap();

        centity_render
    };

    let render = Render {
        manager,
        centity_render,
        init: false,
    };

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
        RESET_FLAG_PRE => {
            manager.on_lost_device();
            crate::external::call_dxreset();
        }

        RESET_FLAG_POST => {
            manager.on_reset_device();
            let rect = crate::utils::client_rect();
            manager.resize(rect[0], rect[1]);
        }
        _ => (),
    }
}

extern "thiscall" fn centity_render(obj: *mut CEntity) {
    let render = Render::get();
    let entity = unsafe { &mut *obj };

    //    if entity.entity_type() == 4 {
    if entity.m_nModelIndex == 3653 {
        let rwobject = entity.rw_entity as *mut RwObject;

        if !rwobject.is_null() {
            unsafe {
                if (*rwobject).obj_type == rpCLUMP {
                    rw::rpclump_for_all_atomics(
                        rwobject as *mut _,
                        Some(atomic_callback),
                        std::ptr::null_mut(),
                    );
                } else {
                    atomic_callback(rwobject as *mut _, std::ptr::null_mut());
                }
            }
        }
        //        }
    }

    render.centity_render.call(obj);
}

use client_api::gta::rw::{self, rpworld::*, rwcore::*, rwplcore::*};
use std::ffi::c_void;

extern "C" fn atomic_callback(atomic: *mut RpAtomic, data: *mut c_void) -> *mut RpAtomic {
    unsafe {
        if !atomic.is_null() && !(*atomic).geometry.is_null() {
            rw::rpgeometry_for_all_materials(
                (*atomic).geometry,
                Some(material_callback),
                std::ptr::null_mut(),
            );
        }
    }

    return atomic;
}

extern "C" fn material_callback(material: *mut RpMaterial, data: *mut c_void) -> *mut RpMaterial {
    unsafe {
        if !(*material).texture.is_null() {
            let name = (*(*material).texture).name();

            let render = Render::get();

            if !render.init {
                render.init = true;
                let raster = &mut *(*(*material).texture).raster;

                let mut manager = render.manager.lock().unwrap();

                manager.create_browser_on_texture(
                    2556,
                    Arc::new(Mutex::new(std::collections::HashMap::new())),
                    "https://www.youtube.com/embed/lWZ-SzhppiM?controls=0&autoplay=1",
                    raster,
                );

                manager.hide_browser(2556, false);
            } else {
                let manager = render.manager.lock().unwrap();
                let texture = manager.raster(2556);

                if !texture.is_null() {
                    (*material).texture = texture;
                }
            }
        }
    }

    return material;
}
