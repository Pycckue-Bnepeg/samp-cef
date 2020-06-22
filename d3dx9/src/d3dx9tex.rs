use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::wingdi::*;
use winapi::um::winnt::*;

use crate::d3dx9core::*;
use crate::d3dx9math::*;
use crate::d3dx9shader::*;

pub mod _D3DXIMAGE_FILEFORMAT {
    pub type Type = i32;
    pub const D3DXIFF_BMP: Type = 0;
    pub const D3DXIFF_JPG: Type = 1;
    pub const D3DXIFF_TGA: Type = 2;
    pub const D3DXIFF_PNG: Type = 3;
    pub const D3DXIFF_DDS: Type = 4;
    pub const D3DXIFF_PPM: Type = 5;
    pub const D3DXIFF_DIB: Type = 6;
    pub const D3DXIFF_HDR: Type = 7;
    pub const D3DXIFF_PFM: Type = 8;
    pub const D3DXIFF_FORCE_DWORD: Type = 2147483647;
}

pub use _D3DXIMAGE_FILEFORMAT::Type as D3DXIMAGE_FILEFORMAT;

pub type LPD3DXFILL2D = Option<
    unsafe extern "stdcall" fn(
        pOut: *mut D3DXVECTOR4,
        pTexCoord: *const D3DXVECTOR2,
        pTexelSize: *const D3DXVECTOR2,
        pData: LPVOID,
    ),
>;
pub type LPD3DXFILL3D = Option<
    unsafe extern "stdcall" fn(
        pOut: *mut D3DXVECTOR4,
        pTexCoord: *const D3DXVECTOR3,
        pTexelSize: *const D3DXVECTOR3,
        pData: LPVOID,
    ),
>;

#[repr(C)]
#[derive(Debug)]
pub struct _D3DXIMAGE_INFO {
    pub Width: UINT,
    pub Height: UINT,
    pub Depth: UINT,
    pub MipLevels: UINT,
    pub Format: D3DFORMAT,
    pub ResourceType: D3DRESOURCETYPE,
    pub ImageFileFormat: D3DXIMAGE_FILEFORMAT,
}
pub type D3DXIMAGE_INFO = _D3DXIMAGE_INFO;

extern "stdcall" {
    pub fn D3DXGetImageInfoFromFileA(pSrcFile: LPCSTR, pSrcInfo: *mut D3DXIMAGE_INFO) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXGetImageInfoFromFileW(pSrcFile: LPCWSTR, pSrcInfo: *mut D3DXIMAGE_INFO) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXGetImageInfoFromResourceA(
        hSrcModule: HMODULE, pSrcResource: LPCSTR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXGetImageInfoFromResourceW(
        hSrcModule: HMODULE, pSrcResource: LPCWSTR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXGetImageInfoFromFileInMemory(
        pSrcData: LPCVOID, SrcDataSize: UINT, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    #[doc = ""]
    pub fn D3DXLoadSurfaceFromFileA(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, pSrcFile: LPCSTR, pSrcRect: *const RECT, Filter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromFileW(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, pSrcFile: LPCWSTR, pSrcRect: *const RECT, Filter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromResourceA(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, hSrcModule: HMODULE, pSrcResource: LPCSTR, pSrcRect: *const RECT,
        Filter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromResourceW(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, hSrcModule: HMODULE, pSrcResource: LPCWSTR, pSrcRect: *const RECT,
        Filter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromFileInMemory(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, pSrcData: LPCVOID, SrcDataSize: UINT, pSrcRect: *const RECT,
        Filter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromSurface(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, pSrcSurface: LPDIRECT3DSURFACE9, pSrcPalette: *const PALETTEENTRY,
        pSrcRect: *const RECT, Filter: DWORD, ColorKey: D3DCOLOR,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadSurfaceFromMemory(
        pDestSurface: LPDIRECT3DSURFACE9, pDestPalette: *const PALETTEENTRY,
        pDestRect: *const RECT, pSrcMemory: LPCVOID, SrcFormat: D3DFORMAT, SrcPitch: UINT,
        pSrcPalette: *const PALETTEENTRY, pSrcRect: *const RECT, Filter: DWORD, ColorKey: D3DCOLOR,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveSurfaceToFileA(
        pDestFile: LPCSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcSurface: LPDIRECT3DSURFACE9,
        pSrcPalette: *const PALETTEENTRY, pSrcRect: *const RECT,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveSurfaceToFileW(
        pDestFile: LPCWSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcSurface: LPDIRECT3DSURFACE9,
        pSrcPalette: *const PALETTEENTRY, pSrcRect: *const RECT,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveSurfaceToFileInMemory(
        ppDestBuf: *mut LPD3DXBUFFER, DestFormat: D3DXIMAGE_FILEFORMAT,
        pSrcSurface: LPDIRECT3DSURFACE9, pSrcPalette: *const PALETTEENTRY, pSrcRect: *const RECT,
    ) -> HRESULT;
}
extern "stdcall" {
    #[doc = ""]
    pub fn D3DXLoadVolumeFromFileA(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        pSrcFile: LPCSTR, pSrcBox: *const D3DBOX, Filter: DWORD, ColorKey: D3DCOLOR,
        pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromFileW(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        pSrcFile: LPCWSTR, pSrcBox: *const D3DBOX, Filter: DWORD, ColorKey: D3DCOLOR,
        pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromResourceA(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        hSrcModule: HMODULE, pSrcResource: LPCSTR, pSrcBox: *const D3DBOX, Filter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromResourceW(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        hSrcModule: HMODULE, pSrcResource: LPCWSTR, pSrcBox: *const D3DBOX, Filter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromFileInMemory(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        pSrcData: LPCVOID, SrcDataSize: UINT, pSrcBox: *const D3DBOX, Filter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromVolume(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        pSrcVolume: LPDIRECT3DVOLUME9, pSrcPalette: *const PALETTEENTRY, pSrcBox: *const D3DBOX,
        Filter: DWORD, ColorKey: D3DCOLOR,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXLoadVolumeFromMemory(
        pDestVolume: LPDIRECT3DVOLUME9, pDestPalette: *const PALETTEENTRY, pDestBox: *const D3DBOX,
        pSrcMemory: LPCVOID, SrcFormat: D3DFORMAT, SrcRowPitch: UINT, SrcSlicePitch: UINT,
        pSrcPalette: *const PALETTEENTRY, pSrcBox: *const D3DBOX, Filter: DWORD,
        ColorKey: D3DCOLOR,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveVolumeToFileA(
        pDestFile: LPCSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcVolume: LPDIRECT3DVOLUME9,
        pSrcPalette: *const PALETTEENTRY, pSrcBox: *const D3DBOX,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveVolumeToFileW(
        pDestFile: LPCWSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcVolume: LPDIRECT3DVOLUME9,
        pSrcPalette: *const PALETTEENTRY, pSrcBox: *const D3DBOX,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveVolumeToFileInMemory(
        ppDestBuf: *mut LPD3DXBUFFER, DestFormat: D3DXIMAGE_FILEFORMAT,
        pSrcVolume: LPDIRECT3DVOLUME9, pSrcPalette: *const PALETTEENTRY, pSrcBox: *const D3DBOX,
    ) -> HRESULT;
}
extern "stdcall" {
    #[doc = ""]
    pub fn D3DXCheckTextureRequirements(
        pDevice: LPDIRECT3DDEVICE9, pWidth: *mut UINT, pHeight: *mut UINT,
        pNumMipLevels: *mut UINT, Usage: DWORD, pFormat: *mut D3DFORMAT, Pool: D3DPOOL,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCheckCubeTextureRequirements(
        pDevice: LPDIRECT3DDEVICE9, pSize: *mut UINT, pNumMipLevels: *mut UINT, Usage: DWORD,
        pFormat: *mut D3DFORMAT, Pool: D3DPOOL,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCheckVolumeTextureRequirements(
        pDevice: LPDIRECT3DDEVICE9, pWidth: *mut UINT, pHeight: *mut UINT, pDepth: *mut UINT,
        pNumMipLevels: *mut UINT, Usage: DWORD, pFormat: *mut D3DFORMAT, Pool: D3DPOOL,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTexture(
        pDevice: LPDIRECT3DDEVICE9, Width: UINT, Height: UINT, MipLevels: UINT, Usage: DWORD,
        Format: D3DFORMAT, Pool: D3DPOOL, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTexture(
        pDevice: LPDIRECT3DDEVICE9, Size: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT,
        Pool: D3DPOOL, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTexture(
        pDevice: LPDIRECT3DDEVICE9, Width: UINT, Height: UINT, Depth: UINT, MipLevels: UINT,
        Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromResourceA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR,
        ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromResourceW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR,
        ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromResourceA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR,
        ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromResourceW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR,
        ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromResourceA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromResourceW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileExA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR, Width: UINT, Height: UINT, MipLevels: UINT,
        Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD, MipFilter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO, pPalette: *mut PALETTEENTRY,
        ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileExW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR, Width: UINT, Height: UINT, MipLevels: UINT,
        Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD, MipFilter: DWORD,
        ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO, pPalette: *mut PALETTEENTRY,
        ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileExA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR, Size: UINT, MipLevels: UINT, Usage: DWORD,
        Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR,
        pSrcInfo: *mut D3DXIMAGE_INFO, pPalette: *mut PALETTEENTRY,
        ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileExW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR, Size: UINT, MipLevels: UINT, Usage: DWORD,
        Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR,
        pSrcInfo: *mut D3DXIMAGE_INFO, pPalette: *mut PALETTEENTRY,
        ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileExA(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCSTR, Width: UINT, Height: UINT, Depth: UINT,
        MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD,
        MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileExW(
        pDevice: LPDIRECT3DDEVICE9, pSrcFile: LPCWSTR, Width: UINT, Height: UINT, Depth: UINT,
        MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD,
        MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromResourceExA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR, Width: UINT,
        Height: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromResourceExW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR, Width: UINT,
        Height: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromResourceExA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR, Size: UINT,
        MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD,
        MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromResourceExW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR, Size: UINT,
        MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD,
        MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromResourceExA(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCSTR, Width: UINT,
        Height: UINT, Depth: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromResourceExW(
        pDevice: LPDIRECT3DDEVICE9, hSrcModule: HMODULE, pSrcResource: LPCWSTR, Width: UINT,
        Height: UINT, Depth: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileInMemory(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT,
        ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileInMemory(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT,
        ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileInMemory(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT,
        ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateTextureFromFileInMemoryEx(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT, Width: UINT,
        Height: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppTexture: *mut LPDIRECT3DTEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateCubeTextureFromFileInMemoryEx(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT, Size: UINT,
        MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL, Filter: DWORD,
        MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppCubeTexture: *mut LPDIRECT3DCUBETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXCreateVolumeTextureFromFileInMemoryEx(
        pDevice: LPDIRECT3DDEVICE9, pSrcData: LPCVOID, SrcDataSize: UINT, Width: UINT,
        Height: UINT, Depth: UINT, MipLevels: UINT, Usage: DWORD, Format: D3DFORMAT, Pool: D3DPOOL,
        Filter: DWORD, MipFilter: DWORD, ColorKey: D3DCOLOR, pSrcInfo: *mut D3DXIMAGE_INFO,
        pPalette: *mut PALETTEENTRY, ppVolumeTexture: *mut LPDIRECT3DVOLUMETEXTURE9,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveTextureToFileA(
        pDestFile: LPCSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcTexture: LPDIRECT3DBASETEXTURE9,
        pSrcPalette: *const PALETTEENTRY,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveTextureToFileW(
        pDestFile: LPCWSTR, DestFormat: D3DXIMAGE_FILEFORMAT, pSrcTexture: LPDIRECT3DBASETEXTURE9,
        pSrcPalette: *const PALETTEENTRY,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXSaveTextureToFileInMemory(
        ppDestBuf: *mut LPD3DXBUFFER, DestFormat: D3DXIMAGE_FILEFORMAT,
        pSrcTexture: LPDIRECT3DBASETEXTURE9, pSrcPalette: *const PALETTEENTRY,
    ) -> HRESULT;
}
extern "stdcall" {
    #[doc = ""]
    pub fn D3DXFilterTexture(
        pBaseTexture: LPDIRECT3DBASETEXTURE9, pPalette: *const PALETTEENTRY, SrcLevel: UINT,
        Filter: DWORD,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillTexture(
        pTexture: LPDIRECT3DTEXTURE9, pFunction: LPD3DXFILL2D, pData: LPVOID,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillCubeTexture(
        pCubeTexture: LPDIRECT3DCUBETEXTURE9, pFunction: LPD3DXFILL3D, pData: LPVOID,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillVolumeTexture(
        pVolumeTexture: LPDIRECT3DVOLUMETEXTURE9, pFunction: LPD3DXFILL3D, pData: LPVOID,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillTextureTX(
        pTexture: LPDIRECT3DTEXTURE9, pTextureShader: LPD3DXTEXTURESHADER,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillCubeTextureTX(
        pCubeTexture: LPDIRECT3DCUBETEXTURE9, pTextureShader: LPD3DXTEXTURESHADER,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXFillVolumeTextureTX(
        pVolumeTexture: LPDIRECT3DVOLUMETEXTURE9, pTextureShader: LPD3DXTEXTURESHADER,
    ) -> HRESULT;
}
extern "stdcall" {
    pub fn D3DXComputeNormalMap(
        pTexture: LPDIRECT3DTEXTURE9, pSrcTexture: LPDIRECT3DTEXTURE9,
        pSrcPalette: *const PALETTEENTRY, Flags: DWORD, Channel: DWORD, Amplitude: FLOAT,
    ) -> HRESULT;
}
