use winapi::shared::d3d9types::*;
use winapi::shared::minwindef::*;

#[repr(C)]
pub struct D3DXMATRIX {
    pub base: D3DMATRIX,
}

pub type LPD3DXMATRIX = *mut D3DXMATRIX;

#[repr(C)]
pub struct D3DXVECTOR2 {
    pub x: FLOAT,
    pub y: FLOAT,
}

pub type LPD3DXVECTOR2 = *mut D3DXVECTOR2;

#[repr(C)]
pub struct D3DXVECTOR3 {
    pub base: D3DVECTOR,
}

impl D3DXVECTOR3 {
    pub fn new(x: f32, y: f32, z: f32) -> D3DXVECTOR3 {
        D3DXVECTOR3 {
            base: D3DVECTOR { x, y, z },
        }
    }
}

pub type LPD3DXVECTOR3 = *mut D3DXVECTOR3;

#[repr(C)]
pub struct D3DXVECTOR4 {
    pub x: FLOAT,
    pub y: FLOAT,
    pub z: FLOAT,
    pub w: FLOAT,
}

pub type LPD3DXVECTOR4 = *mut D3DXVECTOR4;
