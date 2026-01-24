use crate::browser::Browser;

pub trait LifespanHandler {
    fn on_after_created(&self, browser: Browser);
    fn on_before_close(&self, browser: Browser);
}
