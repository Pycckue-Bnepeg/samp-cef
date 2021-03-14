use std::os::raw::c_char;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use cef_api::{cef_list_value_t, List};
use cef_api::{CefApi, InternalApi};

const DLL_PROCESS_ATTACH: u32 = 1;
const DLL_PROCESS_DETACH: u32 = 0;

/// суть приложения заключается в следующем:
/// есть небольшая веб страница с простым интерфейсом. при нажатии на кнопку 0x72 игрок встает на месте,
/// появляется курсор и он должен кликнуть на какого-либо персонажа
/// далее этот плагин ищет игрока, на которого кликнули и отправляет событие обратно в браузер
/// браузер в свою очередь создает кольцо со списком действий над игроком (как пример, показать документы, пожать руку или еще что)
/// при клике на нужное действие отправляется событие уже на сервер, где происходит его обработка
/// браузер сам внутри себя скрывается и обратно разблокирует ввод игроком

/// к сожалению, исходный код той тестовой страницы не сохранился. подразумевалось, что сервер сам создавал браузер как часть своего интерфейса
/// грубо говоря, там был следующий код, который будет в файле ниже

struct App {
    circle: bool,
    pressed: Instant,
    event_tx: Sender<(i32, i32)>,
    event_rx: Receiver<(i32, i32)>,
}

static mut APP: Option<App> = None;
const CEF_INTERFACE_BROWSER: u32 = 102;

#[no_mangle]
pub extern "C" fn cef_initialize(api: *mut InternalApi) {
    CefApi::initialize(api);

    let (event_tx, event_rx) = std::sync::mpsc::channel();

    // подписка на события от браузера (так же можно и от сервера слушать)
    CefApi::subscribe("circle_click", circle_click);
    CefApi::subscribe("circle_closed", circle_closed);

    let app = App {
        circle: false,
        pressed: Instant::now(),
        event_tx,
        event_rx,
    };

    unsafe {
        APP = Some(app);
    }
}

#[no_mangle]
pub extern "C" fn cef_samp_mainloop() {
    if let Some(app) = unsafe { APP.as_mut() } {
        if client_api::utils::is_key_pressed(0x72) {
            if app.pressed.elapsed() >= Duration::from_millis(500) {
                if !app.circle {
                    // если нажата кнопочка и можно в данный момент показать браузер, то показываем его и отсылаем событие
                    if CefApi::try_focus_browser(CEF_INTERFACE_BROWSER) {
                        let args = CefApi::create_list();

                        CefApi::hide_browser(CEF_INTERFACE_BROWSER, false);
                        CefApi::emit_event("show_actions", &args);

                        app.circle = true;
                    }
                } else {
                    CefApi::focus_browser(CEF_INTERFACE_BROWSER, false);
                    CefApi::hide_browser(CEF_INTERFACE_BROWSER, true);
                    app.circle = false;
                }

                app.pressed = Instant::now();
            }
        }

        // если есть новые события от цефа, то обрабатываем их
        while let Ok((x, y)) = app.event_rx.try_recv() {
            let x = x as f32;
            let y = y as f32;

            let mut min = 10000.0f32;
            let mut min_id = u16::max_value();

            // поиск по всем игрокам в зоне стрима нужных, а именно тех, на которых игрок кликнул в браузере
            if let Some(mut players) = client_api::samp::players::players() {
                for player in players.filter(|p| p.is_in_stream()) {
                    if let Some(remote) = player.remote_player() {
                        let pos = remote.position();
                        let (p_x, p_y) = client_api::gta::display::calc_screen_coords(&pos)
                            .unwrap_or((-1.0, -1.0));

                        let delta = ((p_x - x).powf(2.0) + (p_y - y).powf(2.0)).sqrt();

                        if delta < min {
                            min = delta;
                            min_id = remote.id();
                        }
                    }
                }
            }

            if min <= 20.0 {
                if let Some(player) = client_api::samp::players::find_player(min_id as i32)
                    .as_ref()
                    .and_then(|p| p.name())
                {
                    let args = CefApi::create_list();
                    let name = cef::types::string::CefString::new(player);

                    args.set_string(0, &name);
                    args.set_integer(1, min_id as i32);

                    // если игрок найден, отправляет событие обратно в браузер с именем игрока и его ID.
                    CefApi::emit_event("circle_found_player", &args);
                }
            }
        }
    }
}

// событие приходит, когда игрок в браузере нажал ЛКМ в какую-либо точку
// событие содержит в себе координаты экрана X и Y. в браузере выглядит как cef.emit("circle_click", mouse.x, mouse.y);
pub extern "C" fn circle_click(event: *const c_char, args: *mut cef_list_value_t) -> i32 {
    if let Some(args) = List::try_from_raw(args) {
        if args.len() == 3 {
            let x = args.integer(1);
            let y = args.integer(2);

            unsafe {
                APP.as_mut().map(|app| app.event_tx.send((x, y)));
            }
        }
    }

    1
}

pub extern "C" fn circle_closed(_: *const c_char, _: *mut cef_list_value_t) -> i32 {
    if let Some(app) = unsafe { APP.as_mut() } {
        app.circle = false;
    }

    1
}
