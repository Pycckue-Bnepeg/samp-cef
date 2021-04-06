use std::collections::HashMap;

pub struct HudComponent {
    save_origin: bool,
    part_of_interface: bool,
    addr: usize,
    origin_data: Vec<u8>,
    disabled_data: Vec<u8>,
}

impl HudComponent {
    pub fn set_visible(&mut self, visible: bool) {
        unsafe {
            let ptr = self.addr as *mut u8;

            if let Ok(_handle) = region::protect_with_handle(
                ptr,
                self.origin_data.len(),
                region::Protection::READ_WRITE_EXECUTE,
            ) {
                if self.save_origin {
                    for (idx, byte) in self.origin_data.iter_mut().rev().enumerate() {
                        *byte = std::ptr::read(ptr.add(idx));
                    }

                    self.save_origin = false;
                }

                let src = if visible {
                    &self.origin_data
                } else {
                    &self.disabled_data
                };

                for (idx, &byte) in src.iter().rev().enumerate() {
                    std::ptr::write(ptr.add(idx), byte);
                }
            }
        }
    }

    pub fn is_part_of_interface(&self) -> bool {
        self.part_of_interface
    }
}

pub fn make_components() -> HashMap<String, HudComponent> {
    let mut hud = HashMap::new();

    hud.insert(
        "ammo".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x5893B0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "weapon".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x58D7D0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "health".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x589270,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "breath".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x589190,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "armour".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x5890A0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "money".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x58F47D,
            origin_data: vec![0xCC, 0xCC],
            disabled_data: vec![0xE9, 0x90],
        },
    );

    hud.insert(
        "vehicle_name".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x58AEA0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "area_name".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x58AA50,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "radar".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x58A330,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "clock".into(),
        HudComponent {
            save_origin: false,
            part_of_interface: true,
            addr: 0xBAA400,
            origin_data: vec![1],
            disabled_data: vec![0],
        },
    );

    hud.insert(
        "radio".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x4E9E50,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "wanted".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x58D9A0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "crosshair".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x58E020,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "vital_stats".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: true,
            addr: 0x589650,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud.insert(
        "help_text".into(),
        HudComponent {
            save_origin: true,
            part_of_interface: false,
            addr: 0x58B6E0,
            origin_data: vec![0xCC],
            disabled_data: vec![0xC3],
        },
    );

    hud
}
