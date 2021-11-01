//! Application-local prelude: conveniently import types/functions/macros
//! which are generally useful and should be available in every module with
//! `use crate::prelude::*;

/// Abscissa core prelude
pub use abscissa_core::prelude::*;

/// Application state accessors
pub use crate::application::{app_config, app_reader, app_writer};
