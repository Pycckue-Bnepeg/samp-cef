use crate::types::string::CefString;
use crate::v8::V8Value;

pub trait V8Handler {
    fn execute(&self, name: CefString, args: Vec<V8Value>) -> bool;
}
