//! FFI bindings to C++ code.
//! 
//! These types are only meant to be compatible with release builds of
//! software produced by the MSVC compiler.

mod string;
pub use self::string::{String, Str};

mod vector;
pub use self::vector::Vector;
