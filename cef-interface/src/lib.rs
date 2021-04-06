use cef_api::{cef_list_value_t, CefApi, InternalApi, List, CEF_EVENT_BREAK};
use std::collections::HashMap;
use std::os::raw::c_char;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

mod components;

enum Event {
    SetComponentVisible(String, bool),
    PollPlayerStats(bool, i32),
}

struct App {
    hud: HashMap<String, components::HudComponent>,
    polling: bool,
    polling_interval: Duration,
    last_poll: Instant,
    is_hud_visible: bool,
    event_tx: Sender<Event>,
    event_rx: Receiver<Event>,
}

static mut APP: Option<App> = None;

#[no_mangle]
pub unsafe extern "C" fn cef_initialize(api: *mut InternalApi) {
    CefApi::initialize(api);

    let (event_tx, event_rx) = std::sync::mpsc::channel();

    CefApi::subscribe("game:hud:setComponentVisible", set_component_visible);
    CefApi::subscribe("game:data:pollPlayerStats", poll_player_stats);

    let app = App {
        hud: components::make_components(),
        polling: false,
        polling_interval: Duration::from_secs(10),
        last_poll: Instant::now(),
        is_hud_visible: false,
        event_tx,
        event_rx,
    };

    APP = Some(app);
}

#[no_mangle]
pub extern "C" fn cef_samp_mainloop() {
    if let Some(app) = unsafe { APP.as_mut() } {
        while let Ok(event) = app.event_rx.try_recv() {
            match event {
                Event::SetComponentVisible(name, visible) => {
                    if name == "interface" {
                        app.hud
                            .values_mut()
                            .filter(|comp| comp.is_part_of_interface())
                            .for_each(|comp| comp.set_visible(visible));
                    } else {
                        if let Some(component) = app.hud.get_mut(&name) {
                            component.set_visible(visible);
                        }
                    }
                }

                Event::PollPlayerStats(poll, interval) => {
                    app.polling = poll;
                    app.polling_interval = Duration::from_millis(interval as _);

                    let is_hud_visible = client_api::gta::display::is_radar_enabled();
                    update_visible_state(app, is_hud_visible);
                }
            }
        }

        if app.polling && app.last_poll.elapsed() >= app.polling_interval {
            let data = gather_player_data();
            let list = CefApi::create_list();

            list.set_double(0, data.health as _);
            list.set_double(1, data.max_health as _);
            list.set_double(2, data.armour as _);
            list.set_double(3, data.breath as _);
            list.set_integer(4, data.wanted as _);
            list.set_integer(5, data.weapon as _);
            list.set_integer(6, data.ammo as _);
            list.set_integer(7, data.max_ammo as _);
            list.set_integer(8, data.money as _);
            list.set_double(9, data.velocity as _);

            CefApi::emit_event("game:data:playerStats", &list);

            app.last_poll = Instant::now();
        }

        let is_hud_visible = client_api::gta::display::is_radar_enabled();

        if app.polling && is_hud_visible != app.is_hud_visible {
            update_visible_state(app, is_hud_visible);
        }
    }
}

fn update_visible_state(app: &mut App, is_hud_visible: bool) {
    app.is_hud_visible = is_hud_visible;

    let list = CefApi::create_list();
    list.set_bool(0, is_hud_visible);

    CefApi::emit_event("game:hud:newVisibleState", &list);
}

#[no_mangle]
pub extern "C" fn cef_quit() {
    CefApi::uninitialize();

    unsafe {
        APP.take();
    }
}

extern "C" fn set_component_visible(_: *const c_char, args: *mut cef_list_value_t) -> i32 {
    if let Some(args) = List::try_from_raw(args) {
        if args.len() == 3 {
            let name = args.string(1).to_string();
            let visible = args.bool(2);

            unsafe {
                APP.as_mut()
                    .map(|app| app.event_tx.send(Event::SetComponentVisible(name, visible)));
            }
        }
    }

    CEF_EVENT_BREAK
}

extern "C" fn poll_player_stats(_: *const c_char, args: *mut cef_list_value_t) -> i32 {
    if let Some(args) = List::try_from_raw(args) {
        if args.len() == 3 {
            let poll = args.bool(1);
            let interval = args.integer(2);

            unsafe {
                APP.as_mut()
                    .map(|app| app.event_tx.send(Event::PollPlayerStats(poll, interval)));
            }
        }
    }

    CEF_EVENT_BREAK
}

#[derive(Default)]
struct PlayerData {
    health: f32,
    max_health: f32,
    armour: f32,
    breath: f32,
    wanted: u32,
    weapon: u32,
    ammo: u32,
    max_ammo: u32,
    money: i32,
    velocity: f32,
}

#[repr(C)]
struct CWeapon {
    pub r#type: std::os::raw::c_uint,
    pub state: std::os::raw::c_uint,
    pub ammo_in_clip: std::os::raw::c_uint,
    pub total_ammo: std::os::raw::c_uint,
    pub time_for_next_shot: std::os::raw::c_uint,
    pub field_14: std::os::raw::c_char,
    pub field_15: std::os::raw::c_char,
    pub field_16: std::os::raw::c_char,
    pub field_17: std::os::raw::c_char,
    pub fxsystem: *mut (),
}

fn gather_player_data() -> PlayerData {
    let mut data = PlayerData::default();

    unsafe {
        if let Some(local) = client_api::samp::players::local_player() {
            let ped = local.ped() as *const u8;
            let cplayer_data = (ped.add(0x480) as *const *const u8).read();

            data.health = (ped.add(0x540) as *const f32).read();
            data.max_health = (ped.add(0x544) as *const f32).read();
            data.armour = (ped.add(0x548) as *const f32).read();
            data.breath = (cplayer_data.add(0x44) as *const f32).read()
                / get_fat_and_muscule_modifier(STAT_AIR);

            data.wanted = (0x58DB60 as *const u32).read();
            data.money = (0xB7CE50 as *const i32).read();

            let current_slot = ped.add(0x718).read() as usize;
            let weapons = std::slice::from_raw_parts(ped.add(0x5A0) as *mut CWeapon, 13);

            if let Some(weapon) = weapons.get(current_slot) {
                data.weapon = weapon.r#type;
                data.ammo = weapon.ammo_in_clip;
                data.max_ammo = weapon.total_ammo;
            }

            let velo = local.velocity();
            let velo = (velo.x.powi(2) + velo.y.powi(2) + velo.z.powi(2)).sqrt() * 100.0;

            data.velocity = velo;
        }
    }

    data
}

const STAT_AIR: u32 = 8;
const STAT_HP: u32 = 10;

// 8 air
// 10 max hp
fn get_fat_and_muscule_modifier(stat: u32) -> f32 {
    unsafe {
        let f: extern "C" fn(u32) -> f32 = std::mem::transmute(0x559AF0);
        f(stat)
    }
}
