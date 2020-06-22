use crate::types::string::CefString;
use crate::v8::V8Value;
use std::sync::Arc;

pub trait V8Handler {
    fn execute(self: &Arc<Self>, name: CefString, args: Vec<V8Value>) -> bool;
}
