use std::ptr::{null_mut, NonNull};

use d3dx9::d3dx9core::{D3DXCreateSprite, ID3DXSprite, LPD3DXSPRITE};
use d3dx9::d3dx9math::D3DXVECTOR3;

use cef_sys::cef_rect_t;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;

use client_api::gta::rw::rwcore::{RwRaster, RwTexture};

const D3D_OK: i32 = 0;
const D3DXSPRITE_ALPHABLEND: u32 = 16;

pub struct View {
    sprite: Option<NonNull<ID3DXSprite>>,
    texture: Option<NonNull<IDirect3DTexture9>>,
    surface: Option<NonNull<IDirect3DSurface9>>,
    pub rw_texture: Option<NonNull<RwTexture>>,
    rw_raster: Option<NonNull<RwRaster>>,
    width: usize,
    height: usize,
    extern_texture: bool,
}

impl View {
    pub fn new(device: &mut IDirect3DDevice9, width: usize, height: usize) -> View {
        let sprite = Self::create_sprite(device);
        let texture = Self::create_texture(device, width, height);
        let mut surface: *mut IDirect3DSurface9 = std::ptr::null_mut();

        unsafe {
            (*texture).GetSurfaceLevel(0, &mut surface);
        }

        View {
            sprite: NonNull::new(sprite),
            texture: NonNull::new(texture),
            surface: NonNull::new(surface),
            rw_texture: None,
            rw_raster: None,
            extern_texture: false,
            width,
            height,
        }
    }

    pub fn from_extern(origin_raster: &mut RwRaster) -> View {
        let raster = RwRaster::new(origin_raster.width * 5, origin_raster.height * 5);
        let texture = RwTexture::new(raster);

        let mut view = View {
            sprite: None,
            texture: None,
            surface: None,
            rw_texture: NonNull::new(texture),
            rw_raster: NonNull::new(raster),
            extern_texture: true,
            width: (origin_raster.width * 5) as usize,
            height: (origin_raster.height * 5) as usize,
        };

        view
    }

    pub fn is_extern(&self) -> bool {
        self.extern_texture
    }

    pub fn draw(&mut self) {
        if self.is_extern() {
            return;
        }

        unsafe {
            if let Some(sprite) = self.sprite.as_mut().map(|sprite_ptr| sprite_ptr.as_mut()) {
                if let Some(texture) = self
                    .texture
                    .as_mut()
                    .map(|texture_ptr| texture_ptr.as_mut())
                {
                    let device = client_api::gta::d3d9::device();

                    if device.TestCooperativeLevel() == 0 {
                        sprite.Begin(D3DXSPRITE_ALPHABLEND);

                        sprite.Draw(
                            texture,
                            null_mut(),
                            null_mut(),
                            &D3DXVECTOR3::new(0.0, 0.0, 1.0),
                            u32::max_value(),
                        );

                        sprite.End();
                    }
                }
            }
        }
    }

    pub fn update_texture(&mut self, bytes: &[u8], rects: &[cef_rect_t]) {
        unsafe {
            let set_pixels = |dest: *mut u8, pitch: usize| {
                if dest.is_null() {
                    return;
                }

                for cef_rect in rects {
                    for y in cef_rect.y as usize..(cef_rect.y as usize + cef_rect.height as usize) {
                        let index = pitch * y + cef_rect.x as usize * 4;
                        let ptr = dest.add(index);
                        let pixels = bytes.as_ptr();
                        let pixels = pixels.add(index);
                        std::ptr::copy(pixels, ptr, cef_rect.width as usize * 4);
                    }
                }
            };

            if self.is_extern() {
                if let Some(raster) = self.rw_raster.as_mut().map(|ptr| ptr.as_mut()) {
                    let dest = raster.lock(0);
                    let pitch = raster.stride as usize;

                    set_pixels(dest, pitch);

                    raster.unlock();
                }
            } else {
                if let Some(surface) = self.surface.as_mut().map(|ptr| ptr.as_mut()) {
                    let mut rect = D3DLOCKED_RECT {
                        Pitch: 0,
                        pBits: null_mut(),
                    };

                    if surface.LockRect(&mut rect, std::ptr::null(), 0) == D3D_OK {
                        let dest = rect.pBits as *mut u8;
                        let pitch = rect.Pitch as usize;

                        set_pixels(dest, pitch);

                        surface.UnlockRect();
                    }
                }
            }
        }
    }

    pub fn update_popup(&mut self, bytes: &[u8], popup_rect: &cef_rect_t) {
        unsafe {
            if self.is_extern() {
                return;
            }

            if let Some(surface) = self.surface.as_mut().map(|ptr| ptr.as_mut()) {
                let mut rect = D3DLOCKED_RECT {
                    Pitch: 0,
                    pBits: null_mut(),
                };

                if (*surface).LockRect(&mut rect, std::ptr::null(), 0) == D3D_OK {
                    let mut surface_desc: D3DSURFACE_DESC = std::mem::zeroed();

                    surface.GetDesc(&mut surface_desc);

                    let bits = rect.pBits as *mut u8;
                    let pitch = rect.Pitch as usize;

                    let popup_pitch = popup_rect.width * 4;

                    for y in 0..popup_rect.height {
                        let source_index = y * popup_pitch;
                        let dest_index = (y + popup_rect.y) * pitch as i32 + popup_rect.x * 4;

                        let surface_data = bits.add(dest_index as usize);
                        let new_data = bytes.as_ptr().add(source_index as usize);

                        std::ptr::copy(new_data, surface_data, popup_pitch as usize);
                    }

                    (*surface).UnlockRect();
                }
            }
        }
    }

    pub fn rect(&self) -> cef_rect_t {
        cef_rect_t {
            x: 0,
            y: 0,
            width: self.width as _,
            height: self.height as _,
        }
    }

    fn get_rect(&self) -> Option<cef_rect_t> {
        unsafe {
            if let Some(texture) = self
                .texture
                .as_ref()
                .map(|texture_ptr| texture_ptr.as_ref())
            {
                let mut surface_desc: D3DSURFACE_DESC = std::mem::zeroed();

                texture.GetLevelDesc(0, &mut surface_desc);

                let rect = cef_rect_t {
                    x: 0,
                    y: 0,
                    width: surface_desc.Width as _,
                    height: surface_desc.Height as _,
                };

                return Some(rect);
            }
        }

        None
    }

    pub fn on_lost_device(&mut self) {
        unsafe {
            if let Some(mut sprite) = self.sprite.take() {
                sprite.as_mut().Release();
            }

            if let Some(mut surface) = self.surface.take() {
                surface.as_mut().Release();
            }

            if let Some(mut texture) = self.texture.take() {
                texture.as_mut().Release();
            }

            if let Some(mut texture) = self.rw_texture.take() {
                texture.as_mut().destroy();
            }

            if let Some(mut raster) = self.rw_raster.take() {
                raster.as_mut().destroy();
            }
        }
    }

    pub fn clear_texture(&mut self) {
        unsafe {
            if self.is_extern() {
                return;
            } else {
                if let Some(surface) = self.surface.as_ref().map(|ptr| ptr.as_ref()) {
                    let mut rect = D3DLOCKED_RECT {
                        Pitch: 0,
                        pBits: null_mut(),
                    };

                    let mut surface_desc: D3DSURFACE_DESC = std::mem::zeroed();

                    surface.GetDesc(&mut surface_desc);

                    if (*surface).LockRect(&mut rect, std::ptr::null(), D3DLOCK_DISCARD) == D3D_OK {
                        let size = surface_desc.Height as usize * surface_desc.Width as usize * 4;
                        std::ptr::write_bytes(rect.pBits as *mut u8, 0x00, size);

                        (*surface).UnlockRect();
                    }
                }
            }
        }
    }

    pub fn on_reset_device(&mut self, device: &mut IDirect3DDevice9, width: usize, height: usize) {
        if self.extern_texture {
            let raster = RwRaster::new(self.width as i32, self.height as i32);
            let texture = RwTexture::new(raster);

            println!("on_reset_device: {:?} {:?}", raster, texture);

            self.rw_raster = NonNull::new(raster);
            self.rw_texture = NonNull::new(texture);

            return;
        }

        self.sprite = NonNull::new(Self::create_sprite(device));
        self.texture = NonNull::new(Self::create_texture(device, width, height));

        unsafe {
            if let Some(texture) = self.texture.as_mut().map(|a| a.as_mut()) {
                let mut surface: *mut IDirect3DSurface9 = std::ptr::null_mut();

                unsafe {
                    texture.GetSurfaceLevel(0, &mut surface);
                }

                self.surface = NonNull::new(surface);
            }
        }
    }

    fn create_sprite(device: &mut IDirect3DDevice9) -> LPD3DXSPRITE {
        let mut sprite: LPD3DXSPRITE = null_mut();

        unsafe {
            D3DXCreateSprite(device, &mut sprite);
        }

        sprite
    }

    fn create_texture(
        device: &mut IDirect3DDevice9, width: usize, height: usize,
    ) -> LPDIRECT3DTEXTURE9 {
        let mut texture_handle: LPDIRECT3DTEXTURE9 = std::ptr::null_mut();

        unsafe {
            device.CreateTexture(
                width as _,
                height as _,
                1,
                0,
                D3DFMT_A8R8G8B8,
                D3DPOOL_MANAGED,
                &mut texture_handle,
                null_mut(),
            );
        }

        texture_handle
    }
}

impl Drop for View {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut sprite) = self.sprite.take() {
                sprite.as_mut().Release();
            }

            if let Some(mut texture) = self.texture.take() {
                texture.as_mut().Release();
            }
        }
    }
}
