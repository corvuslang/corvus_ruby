#![feature(conservative_impl_trait)] // for the rewrite_error helper and IntoIterator impls

extern crate corvus_core;
#[macro_use]
extern crate lazy_static;
extern crate ruby_sys;
#[macro_use]
extern crate ruru;

mod helpers;
mod emitter;

pub mod error;
pub mod value;
pub mod classes;

#[no_mangle]
pub extern "C" fn initialize_corvus() {
    classes::init();
}
