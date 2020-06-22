use winapi::shared::d3d9::*;
use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;
use winapi::shared::windef::*;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
use winapi::um::winnt::HRESULT;

use winapi::RIDL;

use crate::d3dx9math::*;

RIDL! {#[uuid(0x8ba5fb08, 0x5195, 0x40e2, 0xac, 0x58, 0xd, 0x98, 0x9c, 0x3a, 0x1, 0x2)]
interface ID3DXBuffer(ID3DXBufferVtbl): IUnknown(IUnknownVtbl) {
    fn GetBufferPointer() -> LPVOID,
    fn GetBufferSize() -> DWORD,
}}

pub type LPD3DXBUFFER = *mut ID3DXBuffer;
pub type PD3DXBUFFER = *mut ID3DXBuffer;

RIDL! {#[uuid(0xba0b762d, 0x7d28, 0x43ec, 0xb9, 0xdc, 0x2f, 0x84, 0x44, 0x3b, 0x6, 0x14)]
interface ID3DXSprite(ID3DXSpriteVtbl): IUnknown(IUnknownVtbl) {
    fn GetDevice(ppDevice: *mut LPDIRECT3DDEVICE9,) -> (),
    fn GetTransform(pTransform: *mut D3DXMATRIX,) -> (),
    fn SetTransform(pTransform: *const D3DXMATRIX,) -> (),
    fn SetWorldViewRH(pWorld: *const D3DXMATRIX, pView: *const D3DXMATRIX,) -> (),
    fn SetWorldViewLH(pWorld: *const D3DXMATRIX, pView: *const D3DXMATRIX,) -> (),
    fn Begin(Flags: DWORD,) -> (),
    fn Draw(pTexture: LPDIRECT3DTEXTURE9, pSrcRect: *const RECT, pCenter: *const D3DXVECTOR3, pPosition: *const D3DXVECTOR3, Color: D3DCOLOR,) -> (),
    fn Flush() -> (),
    fn End() -> (),
    fn OnLostDevice() -> (),
    fn OnResetDevice() -> (),
}}

pub type LPD3DXSPRITE = *mut ID3DXSprite;

extern "stdcall" {
    pub fn D3DXCreateSprite(pDevice: LPDIRECT3DDEVICE9, ppSprite: *mut LPD3DXSPRITE) -> HRESULT;
}
