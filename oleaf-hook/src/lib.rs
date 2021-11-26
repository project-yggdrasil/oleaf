//!

#![deny(unsafe_op_in_unsafe_fn, rustdoc::broken_intra_doc_links)]
#![feature(c_size_t)]

#[macro_use]
extern crate static_assertions;

pub mod cxx;

pub mod dml;

pub mod eventhook;

pub mod module;
pub use self::module::Module;
