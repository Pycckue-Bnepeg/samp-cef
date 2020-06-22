use winapi::shared::minwindef::*;
use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};

use winapi::RIDL;

RIDL! {#[uuid(0x3e3d67f8, 0xaa7a, 0x405d, 0xa8, 0x57, 0xba, 0x1, 0xd4, 0x75, 0x84, 0x26)]
interface ID3DXTextureShader(ID3DXTextureShaderVtbl): IUnknown(IUnknownVtbl) {
}}

pub type LPD3DXTEXTURESHADER = *mut ID3DXTextureShader;
