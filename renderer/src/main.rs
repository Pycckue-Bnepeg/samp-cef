#![feature(arbitrary_self_types)]

use winapi::um::libloaderapi::GetModuleHandleA;
use winapi::um::winuser::MessageBoxA;

use cef::app::App;
use cef::browser::{Browser, Frame};
use cef::handlers::render_process::RenderProcessHandler;
use cef::handlers::v8handler::V8Handler;
use cef::types::list::ValueType;
use cef::types::string::CefString;
use cef::v8::{V8Context, V8Value};

use cef::process_message::ProcessMessage;
use cef::types::list::List;
use cef::ProcessId;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type Callbacks = HashMap<String, Vec<V8Value>>;

pub struct Handler {
    frame: Frame,
    subs: Arc<Mutex<Callbacks>>,
}

impl V8Handler for Handler {
    fn execute(self: &Arc<Self>, name: CefString, args: Vec<V8Value>) -> bool {
        let name = name.to_string();

        match name.as_str() {
            "block_input" => {
                let msg = ProcessMessage::create("block_input");
                let list = msg.argument_list();

                if args.len() != 1 {
                    return true;
                }

                convert_to_list(&args, &list);

                self.frame
                    .browser()
                    .main_frame()
                    .send_process_message(ProcessId::Browser, msg);

                return true;
            }

            "on" => {
                if args.len() != 2 {
                    return true;
                }

                let name = args[0].string().to_string();
                let func = args[1].clone();

                let mut events = self.subs.lock().unwrap();

                let subs = events.entry(name).or_insert_with(|| Vec::new());

                subs.push(func);

                return true;
            }

            "emit" => {
                if args.len() < 1 {
                    return false;
                }

                let msg = ProcessMessage::create("emit_event");
                let list = msg.argument_list();

                convert_to_list(&args, &list);

                self.frame
                    .browser()
                    .main_frame()
                    .send_process_message(ProcessId::Browser, msg);
            }

            _ => (),
        }

        false
    }
}

pub struct Application {
    subs: Arc<Mutex<Callbacks>>,
}

impl App for Application {
    type RenderProcessHandler = Self;

    fn render_process_handler(self: &Arc<Self>) -> Option<Arc<Self>> {
        Some(self.clone())
    }
}

impl RenderProcessHandler for Application {
    fn on_context_created(self: &Arc<Self>, _browser: Browser, frame: Frame, context: V8Context) {
        let handler = Arc::new(Handler {
            subs: self.subs.clone(),
            frame,
        });

        let global = context.global();

        let cef_obj = V8Value::new_object();

        let version = V8Value::new_string("0.1.0");
        let func_cur = V8Value::new_function("block_input", Some(handler.clone()));
        let func_on = V8Value::new_function("on", Some(handler.clone()));
        let func_emit = V8Value::new_function("emit", Some(handler));

        let key_str = CefString::new("version");
        let key_func = CefString::new("block_input");
        let key_on = CefString::new("on");
        let key_emit = CefString::new("emit");

        cef_obj.set_value_by_key(&key_str, &version);
        cef_obj.set_value_by_key(&key_func, &func_cur);
        cef_obj.set_value_by_key(&key_on, &func_on);
        cef_obj.set_value_by_key(&key_emit, &func_emit);

        let key_cef = CefString::new("cef");

        global.set_value_by_key(&key_cef, &cef_obj);
    }

    fn on_webkit_initialized(self: &Arc<Self>) {}

    fn on_process_message(
        self: &Arc<Self>, _browser: Browser, frame: Frame, _source: ProcessId, msg: ProcessMessage,
    ) -> bool {
        let name = msg.name().to_string();

        if name == "trigger_event" {
            let args = msg.argument_list();
            let event = args.string(0).to_string();

            let mut params = Vec::with_capacity(args.len() - 1);
            convert_to_v8(&args, 1, &mut params);

            let events = self.subs.lock().unwrap();

            let ctx = frame.context();

            if let Some(subs) = events.get(&event) {
                for func in subs {
                    func.execute_function_with_context(None, &ctx, &params);
                }
            }

            return true;
        }

        false
    }
}

fn main() {
    let instance = unsafe { GetModuleHandleA(std::ptr::null()) };

    let main_args = cef_sys::cef_main_args_t { instance };
    let app = Arc::new(Application {
        subs: Arc::new(Mutex::new(HashMap::new())),
    });

    let code = cef::execute_process(&main_args, Some(app));

    std::process::exit(code);
}

pub fn error_message_box<T: AsRef<str>, M: AsRef<str>>(title: T, message: M) {
    let title = std::ffi::CString::new(title.as_ref()).unwrap();
    let message = std::ffi::CString::new(message.as_ref()).unwrap();
    let flags = winapi::um::winuser::MB_OK | winapi::um::winuser::MB_ICONERROR;

    unsafe {
        MessageBoxA(
            std::ptr::null_mut(),
            message.as_ptr() as *const _,
            title.as_ptr() as *const _,
            flags,
        );
    }
}

fn convert_to_list(v8: &[V8Value], pm: &List) {
    for (idx, value) in v8.iter().enumerate() {
        if value.is_bool() {
            let boolean = value.bool();
            pm.set_bool(idx, boolean);
            continue;
        }

        if value.is_string() {
            let string = value.string();
            pm.set_string(idx, &string);
            continue;
        }

        if value.is_integer() {
            let value = value.integer();
            pm.set_integer(idx, value);
            continue;
        }

        if value.is_double() {
            let value = value.double();
            pm.set_double(idx, value);
            continue;
        }
    }
}

fn convert_to_v8(pm: &List, offset: usize, v8: &mut Vec<V8Value>) {
    for idx in offset..pm.len() {
        match pm.get_type(idx) {
            ValueType::Bool => v8.push(V8Value::new_bool(pm.bool(idx))),
            ValueType::Integer => v8.push(V8Value::new_integer(pm.integer(idx))),
            ValueType::Double => v8.push(V8Value::new_double(pm.double(idx))),
            ValueType::String => v8.push(V8Value::new_cefstring(&pm.string(idx))),
            _ => (),
        }
    }
}
