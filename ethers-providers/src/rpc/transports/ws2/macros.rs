#[macro_export]
macro_rules! ws_error {
    ( $( $t:tt )* ) => {

        #[cfg(target_arch = "wasm32")]
        web_sys::console::error_1(&format!( $( $t )* ).into());
        #[cfg(not(target_arch = "wasm32"))]
        tracing::error!($($t)*);
    }
}

#[macro_export]
macro_rules! ws_warn {
    ( $( $t:tt )* ) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::warn_1(&format!( $( $t )* ).into());
        #[cfg(not(target_arch = "wasm32"))]
        tracing::warn!($($t)*);    }
}

#[macro_export]
macro_rules! ws_debug {
    ( $( $t:tt )* ) => {
        #[cfg(target_arch = "wasm32")]
        web_sys::console::log_1(&format!( $( $t )* ).into());
        #[cfg(not(target_arch = "wasm32"))]
        tracing::debug!($($t)*);    }
}

#[macro_export]
macro_rules! ws_trace {
    ( $( $t:tt )* ) => {
        #[cfg(not(target_arch = "wasm32"))]
        tracing::trace!($($t)*);
    }
}
