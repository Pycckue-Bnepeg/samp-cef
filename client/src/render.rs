use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use winapi::shared::d3d9::IDirect3DDevice9;

use crate::browser::manager::{ExternalClient, Manager};

use client_api::gta::entity::CEntity;
use client_api::gta::menu_manager::CMenuManager;
use client_api::gta::rw::{self, rpworld::*, rwcore::*, rwplcore::*};
use client_api::samp::objects::Object;

use detour::GenericDetour;

const RESET_FLAG_PRE: u8 = 0;
const RESET_FLAG_POST: u8 = 1;

static mut RENDER: Option<Render> = None;

const REFERENCE_FRAMES: u64 = 10;

struct FrameCounter {
    start_at: Instant,
    frames: u64,
    last_fps: u64,
}

struct Render {
    manager: Arc<Mutex<Manager>>,
    centity_render: GenericDetour<extern "thiscall" fn(obj: *mut CEntity)>,
    counter: FrameCounter,
    init: bool,
}

impl Render {
    fn get<'a>() -> Option<&'a mut Render> {
        unsafe { RENDER.as_mut() }
    }

    fn calc_frames(&mut self) -> Option<u64> {
        let counter = &mut self.counter;

        counter.frames += 1;

        if counter.frames == REFERENCE_FRAMES {
            let elapsed = counter.start_at.elapsed().as_millis() as u64;

            let fps = (REFERENCE_FRAMES * 1000) / elapsed;

            counter.last_fps = fps;
            counter.frames = 0;
            counter.start_at = Instant::now();

            return Some(fps);
        }

        None
    }
}

pub fn preinitialize() {
    client_api::gta::d9_proxy::set_proxy(on_create, on_render, on_reset);
}

pub fn initialize(manager: Arc<Mutex<Manager>>) {
    let centity_render = unsafe {
        let render_func: extern "thiscall" fn(*mut CEntity) = std::mem::transmute(0x00534310);
        let centity_render = GenericDetour::new(render_func, centity_render).unwrap();
        centity_render.enable().unwrap();

        centity_render
    };

    let counter = FrameCounter {
        start_at: Instant::now(),
        frames: 0,
        last_fps: 0,
    };

    let render = Render {
        manager,
        centity_render,
        counter,
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
    println!("GTA: Device is created!");
}

fn on_render(_: &mut IDirect3DDevice9) {
    if let Some(render) = Render::get() {
        let fps = render.calc_frames();

        {
            let mut manager = render.manager.lock().unwrap();

            if let Some(&fps) = fps.as_ref() {
                manager.update_fps(fps);
            }

            manager.do_not_draw(CMenuManager::is_menu_active());
            manager.draw();
        }
    }

    crate::app::mainloop();
}

fn on_reset(_: &mut IDirect3DDevice9, reset_flag: u8) {
    if let Some(render) = Render::get() {
        let mut manager = render.manager.lock().unwrap();

        match reset_flag {
            RESET_FLAG_PRE => {
                manager.on_lost_device();
                drop(manager);
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
}

struct RenderState {
    client: *mut ExternalClient,
    before: bool,
}

extern "thiscall" fn centity_render(obj: *mut CEntity) {
    if let Some(render) = Render::get() {
        let mut manager = render.manager.lock().unwrap();
        let entity = unsafe { &mut *obj };

        let browsers = manager.external_browsers();

        for browser in browsers {
            let browser_ptr = browser as *mut _; // должно быть safe
            for &object_id in &browser.object_ids {
                if let Some(object) = Object::get(object_id) {
                    if let Some(obj_entity) = object.entity() {
                        if obj == obj_entity as *mut _ as *mut CEntity {
                            let rwobject = obj_entity._base._base.rw_entity as *mut RwObject;

                            if !rwobject.is_null() {
                                let render_state = Box::new(RenderState {
                                    client: browser_ptr,
                                    before: true,
                                });

                                let render_state = Box::into_raw(render_state) as *mut c_void;

                                replace_texture(rwobject, render_state);

                                render.centity_render.call(obj);

                                let mut render_state =
                                    unsafe { Box::from_raw(render_state as *mut RenderState) };

                                render_state.before = false;

                                let render_state = Box::into_raw(render_state) as *mut c_void;

                                replace_texture(rwobject, render_state);

                                return;
                            }
                        }
                    }
                }
            }
        }

        render.centity_render.call(obj);
    }
}

fn replace_texture(rwobject: *mut RwObject, render_state: *mut c_void) {
    unsafe {
        if (*rwobject).obj_type == rpCLUMP {
            rw::rpclump_for_all_atomics(rwobject as *mut _, Some(atomic_callback), render_state);
        } else {
            atomic_callback(rwobject as *mut _, render_state);
        }
    }
}

extern "C" fn atomic_callback(atomic: *mut RpAtomic, data: *mut c_void) -> *mut RpAtomic {
    unsafe {
        if !atomic.is_null() && !(*atomic).geometry.is_null() {
            let render = Box::from_raw(data as *mut RenderState);

            if render.before {
                before_entity_render(
                    (*(*atomic).geometry).matList.as_mut_slice(),
                    &mut *render.client,
                );
            } else {
                after_entity_render(
                    (*(*atomic).geometry).matList.as_mut_slice(),
                    &mut *render.client,
                );
            }

            Box::into_raw(render);
        }
    }

    return atomic;
}

unsafe fn before_entity_render(materials: &mut [*mut RpMaterial], client: &mut ExternalClient) {
    let mut view = client.browser.view.lock().unwrap();

    for material in materials {
        if !(*material).is_null() {
            let texture = (**material).texture;

            if texture.is_null() {
                continue;
            }

            if !(*texture).name().contains(&client.texture) {
                continue;
            }

            if view.rwtexture().is_none() {
                if !(*texture).raster.is_null() {
                    let raster = &mut *(*texture).raster;
                    let width = (raster.width * client.scale) as usize;
                    let height = (raster.height * client.scale) as usize;

                    drop(view);

                    client.browser.resize(width, height);

                    view = client.browser.view.lock().unwrap();
                }
            }

            if let Some(replace) = view.rwtexture() {
                client.origin_texture = (**material).texture;
                client.prev_replacement = replace.as_ptr();
                (**material).set_texture(replace.as_ptr());
            }
        }
    }
}

unsafe fn after_entity_render(materials: &mut [*mut RpMaterial], client: &mut ExternalClient) {
    for material in materials {
        if !(*material).is_null() {
            let texture = (**material).texture;

            if texture.is_null() || texture != client.prev_replacement {
                continue;
            }

            (**material).set_texture(client.origin_texture);
        }
    }
}
