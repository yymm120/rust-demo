use wasm_bindgen::JsValue;




pub(crate) trait UnwrapJsExt<T> {
    fn unwrap_js(self) -> T;
}


#[cfg(debug_assertions)]
impl<T> UnwrapJsExt<T> for Result<T, JsValue> {
    fn unwrap_js(self) -> T {
        match self {
            Ok(value) => value,
            Err(e) => {
                use wasm_bindgen::JsCast;

                match e.dyn_ref::<js_sys::Error>() {
                    Some(e) => {
                        panic!("{}", e.message());
                    },
                    None => {
                        panic!("{:?}", e);
                    }
                }
            }
        }
    }
}


#[macro_export]
macro_rules! __unwrap {
    ($value:expr, $var:ident => $error:expr,) => {{
        match $value {
            Ok(value) => value,
            Err($var) => $error,
        }
        $value.unwrap_throw()
    }};
}