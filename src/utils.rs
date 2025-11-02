use wasm_bindgen::prelude::*;

pub(crate) fn log(x: &str) {
    web_sys::console::log_1(&JsValue::from_str(x));
}

#[macro_export]
macro_rules! log {
    ($($expr:tt)*) => {
        {
            use std::fmt::Write;
            let mut s = String::new();
            let _ = write!(&mut s, $($expr)*);
            crate::utils::log(&s);
        }
    };
}
