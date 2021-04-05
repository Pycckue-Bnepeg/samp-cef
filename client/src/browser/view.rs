use crate::utils::RenderMode;
use cef_sys::cef_rect_t;
use client_api::gta::matrix::CRect;
use client_api::gta::rw::rwcore::{RwRaster, RwTexture};
use client_api::gta::rw::rwplcore::RwRGBA;
use client_api::gta::sprite::Sprite;
use d3dx9::d3dx9core::{D3DXCreateSprite, ID3DXSprite};
use d3dx9::d3dx9math::D3DXVECTOR3;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use winapi::shared::d3d9::{IDirect3DDevice9, IDirect3DSurface9, IDirect3DTexture9};
use winapi::shared::d3d9types::{
    D3DFMT_A8R8G8B8, D3DLOCKED_RECT, D3DLOCK_DISCARD, D3DPOOL_DEFAULT, D3DPOOL_MANAGED,
    D3DSURFACE_DESC, D3DUSAGE_DYNAMIC, D3DUSAGE_WRITEONLY,
};

const D3D_OK: i32 = 0;
const D3DXSPRITE_ALPHABLEND: u32 = 16;

macro_rules! set_texture_bytes {
    ($s:ident, $dest:ident, $pitch:ident, $body:block) => {
        if let Some(mut $dest) = $s.rw_sprite.as_mut().and_then(|rw| rw.bytes()) {
            let $pitch = $dest.pitch;
            let $dest = &mut *$dest;
            $body

            return;
        }

        if let Some(mut $dest) = $s.directx.as_mut().and_then(|d3d9| d3d9.bytes()) {
            let $pitch = $dest.pitch;
            let $dest = &mut *$dest;
            $body

            return;
        }

        if let Some(mut $dest) = $s.renderware.as_mut().and_then(|rw| rw.bytes()) {
            let $pitch = $dest.pitch;
            let $dest = &mut *$dest;
            $body

            return;
        }
    };
}

pub struct D3LockGuard<'a> {
    bytes: &'a mut [u8],
    pub pitch: usize,
    surface: NonNull<IDirect3DSurface9>,
}

impl D3LockGuard<'_> {
    #[inline(always)]
    pub fn bytes_as_mut_ptr(&mut self) -> *mut u8 {
        self.bytes.as_mut_ptr()
    }
}

impl Deref for D3LockGuard<'_> {
    type Target = [u8];

    #[inline(always)]
    fn deref(&self) -> &[u8] {
        self.bytes
    }
}

impl DerefMut for D3LockGuard<'_> {
    #[inline(always)]
    fn deref_mut(&mut self) -> &mut [u8] {
        self.bytes
    }
}

impl Drop for D3LockGuard<'_> {
    fn drop(&mut self) {
        unsafe {
            self.surface.as_mut().UnlockRect();
        }
    }
}

pub struct D3Container {
    sprite: Option<NonNull<ID3DXSprite>>,
    texture: Option<NonNull<IDirect3DTexture9>>,
    surface: Option<NonNull<IDirect3DSurface9>>,
}

impl D3Container {
    pub fn new(device: &mut IDirect3DDevice9, width: usize, height: usize) -> D3Container {
        let mut sprite = std::ptr::null_mut();
        let mut texture = std::ptr::null_mut();
        let mut surface = std::ptr::null_mut();

        unsafe {
            D3DXCreateSprite(device, &mut sprite);

            device.CreateTexture(
                width as _,
                height as _,
                1,
                D3DUSAGE_DYNAMIC,
                D3DFMT_A8R8G8B8,
                D3DPOOL_DEFAULT,
                &mut texture,
                std::ptr::null_mut(),
            );

            (*texture).GetSurfaceLevel(0, &mut surface);
        }

        D3Container {
            sprite: NonNull::new(sprite),
            texture: NonNull::new(texture),
            surface: NonNull::new(surface),
        }
    }

    #[inline]
    pub fn draw(&mut self) {
        unsafe {
            if let Some(sprite) = self.sprite.as_mut().map(|s| s.as_mut()) {
                if let Some(texture) = self.texture.as_mut().map(|t| t.as_mut()) {
                    sprite.Begin(D3DXSPRITE_ALPHABLEND);

                    sprite.Draw(
                        texture,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &D3DXVECTOR3::new(0.0, 0.0, 1.0),
                        u32::max_value(),
                    );

                    sprite.End();
                }
            }
        }
    }

    #[inline]
    pub fn bytes(&mut self) -> Option<D3LockGuard> {
        unsafe {
            self.surface.as_mut().and_then(|surface| {
                let mut rect = D3DLOCKED_RECT {
                    Pitch: 0,
                    pBits: std::ptr::null_mut(),
                };

                let mut desc: D3DSURFACE_DESC = std::mem::zeroed();

                surface.as_mut().GetDesc(&mut desc);

                if surface
                    .as_mut()
                    .LockRect(&mut rect, std::ptr::null(), 0) // discard is a good choice
                    == D3D_OK
                    && !rect.pBits.is_null()
                {
                    let size = desc.Width * desc.Height * 4;
                    Some(D3LockGuard {
                        bytes: std::slice::from_raw_parts_mut(rect.pBits as *mut u8, size as usize),
                        pitch: rect.Pitch as usize,
                        surface: surface.clone(),
                    })
                } else {
                    None
                }
            })
        }
    }
}

impl Drop for D3Container {
    fn drop(&mut self) {
        unsafe {
            if let Some(mut surface) = self.surface.take() {
                surface.as_mut().Release();
            }

            if let Some(mut texture) = self.texture.take() {
                texture.as_mut().Release();
            }

            if let Some(mut sprite) = self.sprite.take() {
                sprite.as_mut().Release();
            }
        }
    }
}

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
                    raster: raster.clone(),
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
    sprite: Option<Sprite>,
    rw: RwContainer,
}

impl SpriteContainer {
    pub fn new(width: usize, height: usize) -> SpriteContainer {
        let rw = RwContainer::new(width, height);
        let mut sprite = Sprite::new();
        sprite.set_texture(rw.texture.unwrap().as_ptr());

        SpriteContainer {
            sprite: Some(sprite),
            rw,
        }
    }

    #[inline]
    pub fn draw(&mut self) {
        if let Some(sprite) = self.sprite.as_mut() {
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

            sprite.draw(rect, color);
        }
    }

    #[inline]
    pub fn bytes(&mut self) -> Option<RwLockGuard> {
        self.rw.bytes()
    }
}

pub struct View {
    directx: Option<D3Container>,
    rw_sprite: Option<SpriteContainer>,
    renderware: Option<RwContainer>,
    width: usize,
    height: usize,
    render_mode: RenderMode,
}

impl View {
    pub fn new(render_mode: RenderMode) -> View {
        View {
            directx: None,
            rw_sprite: None,
            renderware: None,
            width: 0,
            height: 0,
            render_mode,
        }
    }

    pub fn make_directx(&mut self, device: &mut IDirect3DDevice9, width: usize, height: usize) {
        let width = std::cmp::max(1, width);
        let height = std::cmp::max(1, height);

        self.destroy_previous();

        match self.render_mode {
            RenderMode::Renderware => self.rw_sprite = Some(SpriteContainer::new(width, height)),
            RenderMode::DirectX => self.directx = Some(D3Container::new(device, width, height)),
            RenderMode::Empty => (),
        }

        self.set_size(width, height);
    }

    pub fn make_renderware(&mut self, raster: &mut RwRaster, scale: i32) {
        let width = std::cmp::max(1, (raster.width * scale) as usize);
        let height = std::cmp::max(1, (raster.height * scale) as usize);

        self.destroy_previous();

        let container = RwContainer::new(width, height);

        self.renderware = Some(container);
        self.set_size(width, height);
    }

    #[inline(never)]
    pub fn make_empty(&mut self) {
        self.destroy_previous();
        self.render_mode = RenderMode::Empty;
        self.set_size(1, 1);
    }

    #[inline]
    pub fn draw(&mut self) {
        self.rw_sprite.as_mut().map(|rw| rw.draw());
        self.directx.as_mut().map(|d3d9| d3d9.draw());
    }

    #[inline(always)]
    pub fn update_texture(&mut self, bytes: &[u8], rects: &[cef_rect_t]) {
        set_texture_bytes!(self, dest, pitch, {
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
        });
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

    pub fn resize(&mut self, device: Option<&mut IDirect3DDevice9>, width: usize, height: usize) {
        if self.render_mode == RenderMode::Empty {
            return;
        }

        let should_replace =
            (device.is_some() && self.directx.is_none() && self.rw_sprite.is_none())
                || (device.is_none() && self.renderware.is_none());

        if self.width == width && self.height == height && !should_replace {
            return;
        }

        let width = std::cmp::max(1, width);
        let height = std::cmp::max(1, height);

        self.destroy_previous();
        self.set_size(width, height);

        if let Some(device) = device {
            match self.render_mode {
                RenderMode::DirectX => self.directx = Some(D3Container::new(device, width, height)),
                RenderMode::Renderware => {
                    self.rw_sprite = Some(SpriteContainer::new(width, height))
                }
                _ => (),
            }
        } else {
            self.renderware = Some(RwContainer::new(width, height));
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
        self.renderware.as_mut().and_then(|rw| rw.texture.clone())
    }

    pub fn is_empty(&self) -> bool {
        self.render_mode == RenderMode::Empty
    }

    pub fn set_render_mode(&mut self, render_mode: RenderMode) {
        self.render_mode = render_mode;
    }

    fn destroy_previous(&mut self) {
        self.rw_sprite.take();
        self.directx.take();
        self.renderware.take();
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
        if let Some(mut bytes) = self.rw_sprite.as_mut().and_then(|rw| rw.bytes()) {
            let pitch = bytes.pitch;
            func(&mut *bytes, pitch);

            return;
        }

        if let Some(mut bytes) = self.directx.as_mut().and_then(|d3d9| d3d9.bytes()) {
            let pitch = bytes.pitch;
            func(&mut *bytes, pitch);

            return;
        }

        if let Some(mut bytes) = self.renderware.as_mut().and_then(|rw| rw.bytes()) {
            let pitch = bytes.pitch;
            func(&mut *bytes, pitch);

            return;
        }
    }
}
