//!

#![deny(unsafe_op_in_unsafe_fn, rustdoc::broken_intra_doc_links)]
#![feature(arbitrary_enum_discriminant, c_size_t)]

#[macro_use]
extern crate static_assertions;

pub use oleaf_hook_macros::*;

pub mod cxx;

pub mod dml;

pub mod event;

pub mod module;
pub use self::module::Module;
