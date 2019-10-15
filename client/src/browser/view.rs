use std::ptr::{null_mut, NonNull};

use d3dx9::d3dx9core::{D3DXCreateSprite, ID3DXSprite, LPD3DXSPRITE};
use d3dx9::d3dx9math::D3DXVECTOR3;

use cef_sys::cef_rect_t;

use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;

const D3D_OK: i32 = 0;
const D3DXSPRITE_ALPHABLEND: u32 = 16;

pub struct View {
    sprite: Option<NonNull<ID3DXSprite>>,
    texture: Option<NonNull<IDirect3DTexture9>>,
    width: usize,
    height: usize,
    extern_texture: bool,
}

impl View {
    pub fn new(device: &mut IDirect3DDevice9, width: usize, height: usize) -> View {
        let sprite = Self::create_sprite(device);
        let texture = Self::create_texture(device, width, height);

        View {
            sprite: NonNull::new(sprite),
            texture: NonNull::new(texture),
            extern_texture: false,
            width,
            height,
        }
    }

    pub fn from_extern(texture: *mut IDirect3DTexture9) -> View {
        let mut view = View {
            sprite: None,
            texture: NonNull::new(texture),
            extern_texture: true,
            width: 0,
            height: 0,
        };

        unsafe {
            if let Some(texture) = view.texture.as_mut().map(|txt| txt.as_mut()) {
                texture.AddRef();
            }
        }

        if let Some(rect) = view.get_rect() {
            view.width = rect.width as usize;
            view.height = rect.height as usize;
        }

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
            if let Some(texture) = self
                .texture
                .as_mut()
                .map(|texture_ptr| texture_ptr.as_mut())
            {
                let mut rect = D3DLOCKED_RECT {
                    Pitch: 0,
                    pBits: null_mut(),
                };

                let mut surface_desc: D3DSURFACE_DESC = std::mem::zeroed();

                texture.GetLevelDesc(0, &mut surface_desc);

                if (*texture).LockRect(0, &mut rect, std::ptr::null(), 0) == D3D_OK {
                    let bits = rect.pBits as *mut u8;
                    let pitch = rect.Pitch as usize;

                    for cef_rect in rects {
                        for y in
                            cef_rect.y as usize..(cef_rect.y as usize + cef_rect.height as usize)
                        {
                            let index = pitch * y + cef_rect.x as usize * 4;
                            let ptr = bits.add(index);
                            let pixels = bytes.as_ptr();
                            let pixels = pixels.add(index);
                            std::ptr::copy(pixels, ptr, cef_rect.width as usize * 4);
                        }
                    }

                    (*texture).UnlockRect(0);
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

    pub fn buffer(&self) -> Option<Vec<u8>> {
        unsafe {
            if let Some(texture) = self
                .texture
                .as_ref()
                .map(|texture_ptr| texture_ptr.as_ref())
            {
                let mut rect = D3DLOCKED_RECT {
                    Pitch: 0,
                    pBits: null_mut(),
                };

                let mut surface_desc: D3DSURFACE_DESC = std::mem::zeroed();

                texture.GetLevelDesc(0, &mut surface_desc);

                if (*texture).LockRect(0, &mut rect, std::ptr::null(), 0) == D3D_OK {
                    let size = surface_desc.Height as usize * surface_desc.Width as usize * 4;
                    let buffer = std::slice::from_raw_parts(rect.pBits as *mut u8, size);
                    let buffer = Vec::from(buffer);

                    (*texture).UnlockRect(0);

                    return Some(buffer);
                }
            }
        }

        return None;
    }

    pub fn on_lost_device(&mut self) {
        unsafe {
            if let Some(mut sprite) = self.sprite.take() {
                sprite.as_mut().Release();
            }

            if let Some(mut texture) = self.texture.take() {
                texture.as_mut().Release();
            }
        }
    }

    pub fn on_reset_device(&mut self, device: &mut IDirect3DDevice9, width: usize, height: usize) {
        unsafe {
            self.sprite = NonNull::new(Self::create_sprite(device));
            self.texture = NonNull::new(Self::create_texture(device, width, height));
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
