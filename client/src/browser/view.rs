use cef_sys::cef_rect_t;
use client_api::gta::matrix::CRect;
use client_api::gta::rw;
use client_api::gta::rw::rwcore::{RwRaster, RwTexture};
use client_api::gta::rw::rwplcore::{self, RwRGBA};
use client_api::gta::sprite::Sprite;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;

pub struct RwLockGuard<'a> {
    bytes: &'a mut [u8],
    pub pitch: usize,
    raster: NonNull<RwRaster>,
}

impl RwLockGuard<'_> {
    #[inline(always)]
    pub fn bytes_as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }
}

impl Deref for RwLockGuard<'_> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.bytes
    }
}

impl DerefMut for RwLockGuard<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.bytes
    }
}

impl Drop for RwLockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.raster.as_mut().unlock();
        }
    }
}

pub struct RwContainer {
    texture: Option<NonNull<RwTexture>>,
    raster: Option<NonNull<RwRaster>>,
}

impl RwContainer {
    pub fn new(width: usize, height: usize) -> RwContainer {
        let raster = RwRaster::new(width as i32, height as i32);
        let texture = RwTexture::new(raster);

        RwContainer {
            texture: NonNull::new(texture),
            raster: NonNull::new(raster),
        }
    }

    #[inline]
    pub fn bytes(&mut self) -> Option<RwLockGuard> {
        unsafe {
            self.raster.as_mut().map(|raster| {
                let bytes = raster.as_mut().lock(0);
                let size = {
                    let raster = raster.as_mut();
                    raster.height * raster.width * 4
                };

                RwLockGuard {
                    bytes: std::slice::from_raw_parts_mut(bytes, size as usize),
                    pitch: raster.as_mut().stride as usize,
                    raster: *raster,
                }
            })
        }
    }
}

impl Drop for RwContainer {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut texture) = self.texture.take() {
                texture.as_mut().destroy();
            }

            if let Some(mut raster) = self.raster.take() {
                raster.as_mut().destroy();
            }
        }
    }
}

pub struct SpriteContainer {
    sprite: Sprite,
    rw: RwContainer,
}

impl SpriteContainer {
    pub fn new(width: usize, height: usize) -> SpriteContainer {
        let rw = RwContainer::new(width, height);
        let mut sprite = Sprite::new();
        sprite.set_texture(rw.texture.unwrap().as_ptr());

        SpriteContainer { sprite, rw }
    }

    #[inline]
    pub fn draw(&mut self) {
        let client = crate::utils::client_rect();
        let rect = CRect {
            top: 0.0,
            left: 0.0,
            right: client[0] as f32,
            bottom: client[1] as f32,
        };

        let color = RwRGBA {
            red: 0xFF,
            green: 0xFF,
            blue: 0xFF,
            alpha: 0xFF,
        };

        let prev = rw::render_state(rwplcore::RENDERSTATETEXTUREFILTER);

        rw::set_render_state(rwplcore::RENDERSTATETEXTUREFILTER, rwplcore::FILTERNEAREST);
        self.sprite.draw(rect, color);

        rw::set_render_state(rwplcore::RENDERSTATETEXTUREFILTER, prev);
    }

    #[inline]
    pub fn bytes(&mut self) -> Option<RwLockGuard> {
        self.rw.bytes()
    }
}

enum ViewContainer {
    Material(RwContainer),
    Display(SpriteContainer),
}

impl ViewContainer {
    fn draw(&mut self) {
        match self {
            ViewContainer::Display(sprite) => sprite.draw(),
            _ => (),
        }
    }

    fn bytes(&mut self) -> Option<RwLockGuard> {
        match self {
            ViewContainer::Display(sprite) => sprite.bytes(),
            ViewContainer::Material(sprite) => sprite.bytes(),
        }
    }

    fn texture(&self) -> Option<NonNull<RwTexture>> {
        match self {
            ViewContainer::Display(sprite) => sprite.rw.texture.clone(),
            ViewContainer::Material(rw) => rw.texture.clone(),
        }
    }
}

pub struct View {
    container: Option<ViewContainer>,
    width: usize,
    height: usize,
    active: bool,
}

impl View {
    pub fn new() -> View {
        View {
            container: None,
            width: 0,
            height: 0,
            active: true,
        }
    }

    pub fn make_display(&mut self, width: usize, height: usize) {
        let width = std::cmp::max(1, width);
        let height = std::cmp::max(1, height);

        self.destroy_previous();

        self.container = Some(ViewContainer::Display(SpriteContainer::new(width, height)));

        self.set_size(width, height);
    }

    #[inline(never)]
    pub fn make_inactive(&mut self) {
        self.destroy_previous();
        self.set_size(1, 1);
        self.active = false;
    }

    pub fn make_active(&mut self) {
        self.active = true;
    }

    #[inline]
    pub fn draw(&mut self) {
        if let Some(rw) = self.container.as_mut() {
            rw.draw()
        }
    }

    #[inline(always)]
    pub fn update_texture(&mut self, bytes: &[u8], rects: &[cef_rect_t]) {
        if let Some(mut dest) = self.container.as_mut().and_then(|rw| rw.bytes()) {
            let pitch = dest.pitch;
            let dest = &mut *dest;

            let dest = dest.as_mut_ptr();
            let pixels_origin = bytes.as_ptr();

            for cef_rect in rects {
                for y in cef_rect.y as usize..(cef_rect.y as usize + cef_rect.height as usize) {
                    unsafe {
                        let index = pitch * y + cef_rect.x as usize * 4;
                        let ptr = dest.add(index);
                        let pixels = pixels_origin.add(index);
                        std::ptr::copy(pixels, ptr, cef_rect.width as usize * 4);
                    }
                }
            }
        }
    }

    pub fn update_popup(&mut self, bytes: &[u8], popup_rect: &cef_rect_t) {
        let set_pixels = |dest: &mut [u8], pitch: usize| {
            let dest = dest.as_mut_ptr();
            let popup_pitch = popup_rect.width * 4;

            for y in 0..popup_rect.height {
                let source_index = y * popup_pitch;
                let dest_index = (y + popup_rect.y) * pitch as i32 + popup_rect.x * 4;

                unsafe {
                    let surface_data = dest.add(dest_index as usize);
                    let new_data = bytes.as_ptr().add(source_index as usize);

                    std::ptr::copy(new_data, surface_data, popup_pitch as usize);
                }
            }
        };

        self.set_texture_bytes(set_pixels);
    }

    pub fn clear(&mut self) {
        let clear = |dest: &mut [u8], _: usize| {
            let size = dest.len();
            let dest = dest.as_mut_ptr();

            unsafe {
                std::ptr::write_bytes(dest, 0x00, size);
            }
        };

        self.set_texture_bytes(clear);
    }

    pub fn on_lost_device(&mut self) {
        self.destroy_previous();
    }

    pub fn resize(&mut self, is_object: bool, width: usize, height: usize) {
        if !self.active {
            return;
        }

        let should_replace = self.active && self.container.is_none();

        if self.width == width && self.height == height && !should_replace {
            return;
        }

        let width = std::cmp::max(1, width);
        let height = std::cmp::max(1, height);

        self.destroy_previous();
        self.set_size(width, height);

        if is_object {
            self.container = Some(ViewContainer::Material(RwContainer::new(width, height)));
        } else {
            self.container = Some(ViewContainer::Display(SpriteContainer::new(width, height)));
        }
    }

    pub fn rect(&self) -> cef_rect_t {
        let width = if self.width == 0 {
            1
        } else {
            self.width as i32
        };

        let height = if self.height == 0 {
            1
        } else {
            self.height as i32
        };

        cef_rect_t {
            width,
            height,
            x: 0,
            y: 0,
        }
    }

    pub fn rwtexture(&mut self) -> Option<NonNull<RwTexture>> {
        self.container.as_mut().and_then(|rw| rw.texture())
    }

    pub fn is_empty(&self) -> bool {
        // self.render_mode == RenderMode::Empty
        false
    }

    fn destroy_previous(&mut self) {
        self.container.take();
    }

    fn set_size(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
    }

    #[inline(always)]
    fn set_texture_bytes<F>(&mut self, mut func: F)
    where
        F: FnMut(&mut [u8], usize),
    {
        if let Some(mut bytes) = self.container.as_mut().and_then(|rw| rw.bytes()) {
            let pitch = bytes.pitch;
            func(&mut *bytes, pitch);
        }
    }
}
