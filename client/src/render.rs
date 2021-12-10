use std::ffi::c_void;
use std::sync::Arc;
use std::time::Instant;

use parking_lot::Mutex;

use crate::browser::manager::{ExternalClient, Manager};

use client_api::gta::entity::CEntity;
use client_api::gta::menu_manager::CMenuManager;
use client_api::gta::rw::{self, rpworld::*, rwplcore::*};
use client_api::samp::objects::Object;

use detour::GenericDetour;

static mut RENDER: Option<Render> = None;

const REFERENCE_FRAMES: u64 = 10;

const DRAWING_EVENT: usize = 0x58FAE0;
const SHUTDOWN_RW_EVENT: usize = 0x53BB80;

type DrawingEventFn = extern "C" fn();
type ShutdownRwEventFn = extern "C" fn();

struct FrameCounter {
    start_at: Instant,
    frames: u64,
    last_fps: u64,
}

struct Render {
    manager: Arc<Mutex<Manager>>,
    centity_render: GenericDetour<extern "thiscall" fn(obj: *mut CEntity)>,
    drawing_event: GenericDetour<DrawingEventFn>,
    shutdown_event: GenericDetour<ShutdownRwEventFn>,
    counter: FrameCounter,
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

pub fn initialize(manager: Arc<Mutex<Manager>>) {
    log::trace!("hooking CEntity::render");

    let centity_render = unsafe {
        let render_func: extern "thiscall" fn(*mut CEntity) = std::mem::transmute(0x00534310);
        let centity_render = GenericDetour::new(render_func, centity_render).unwrap();

        centity_render.enable().unwrap();
        centity_render
    };

    let drawing_event = unsafe {
        let func: DrawingEventFn = std::mem::transmute(DRAWING_EVENT);
        let hook = GenericDetour::new(func, drawing_event).unwrap();

        hook.enable().unwrap();
        hook
    };

    let shutdown_event = unsafe {
        let func: ShutdownRwEventFn = std::mem::transmute(SHUTDOWN_RW_EVENT);
        let hook = GenericDetour::new(func, shutdown_event).unwrap();

        hook.enable().unwrap();
        hook
    };

    log::trace!("hooking ok ...");

    let counter = FrameCounter {
        start_at: Instant::now(),
        frames: 0,
        last_fps: 0,
    };

    let render = Render {
        manager,
        centity_render,
        drawing_event,
        shutdown_event,
        counter,
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

fn on_render() {
    crate::app::mainloop();
}

pub fn render() {
    if let Some(render) = Render::get() {
        let fps = render.calc_frames();

        {
            let mut manager = render.manager.lock();

            if let Some(&fps) = fps.as_ref() {
                manager.update_fps(fps);
            }

            manager.do_not_draw(CMenuManager::is_menu_active());
            manager.draw();
        }
    }
}

fn on_destroy() {
    log::trace!("rwshutdown ...");

    if let Some(render) = Render::get() {
        let mut manager = render.manager.lock();
        manager.remove_views();
    }
}

struct RenderState {
    client: *mut ExternalClient,
    before: bool,
}

extern "C" fn drawing_event() {
    if let Some(render) = Render::get() {
        render.drawing_event.call();
    }

    on_render();
}

extern "C" fn shutdown_event() {
    on_destroy();

    if let Some(render) = Render::get() {
        render.shutdown_event.call();
    }
}

extern "thiscall" fn centity_render(obj: *mut CEntity) {
    if let Some(render) = Render::get() {
        let mut manager = render.manager.lock();
        let _entity = unsafe { &mut *obj };

        let browsers = manager.external_browsers();

        for browser in browsers {
            let browser_ptr = browser as *mut _; // должно быть safe
            for &object_id in &browser.object_ids {
                if let Some(object) = Object::get(object_id) {
                    if let Some(obj_entity) = object.entity() {
                        if obj == obj_entity as *mut _ as *mut CEntity {
                            let rwobject = obj_entity._base._base.rw_entity as *mut RwObject;

                            if !rwobject.is_null() {
                                let mut render_state = RenderState {
                                    client: browser_ptr,
                                    before: true,
                                };

                                let render_ptr = &mut render_state as *mut _ as *mut c_void;

                                replace_texture(rwobject, render_ptr);

                                render.centity_render.call(obj);

                                render_state.before = false;

                                replace_texture(rwobject, render_ptr);

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
            let render = &mut *(data as *mut RenderState);

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
        }
    }

    atomic
}

unsafe fn before_entity_render(materials: &mut [*mut RpMaterial], client: &mut ExternalClient) {
    for material in materials {
        if !(*material).is_null() {
            let texture = (**material).texture;

            if texture.is_null() {
                continue;
            }

            if !(*texture).name().contains(&client.texture) {
                continue;
            }

            let mut view = client.browser.view.lock();

            if view.rwtexture().is_none() && !(*texture).raster.is_null() {
                let raster = &mut *(*texture).raster;
                let width = (raster.width * client.scale) as usize;
                let height = (raster.height * client.scale) as usize;

                view.make_active();

                drop(view);

                client.browser.resize(width, height);
                client.browser.restore_hide_status();

                view = client.browser.view.lock();
            }

            if let Some(replace) = view.rwtexture() {
                client.origin_surface_props = (**material).surface_props.clone();

                (**material).surface_props.ambient = 16.0;
                (**material).surface_props.diffuse = 0.0;
                (**material).surface_props.specular = 0.0;

                client.origin_texture = (**material).texture;
                client.prev_replacement = replace.as_ptr();
                (**material).texture = replace.as_ptr();

                break; // replaced. do not replace another
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

            (**material).texture = client.origin_texture;
            (**material).surface_props = client.origin_surface_props.clone();

            break; //
        }
    }
}
