//! ABI-compatible types depicting relevant primitives of the DML system.

use std::os::raw::{c_double, c_float, c_int, c_ulonglong, c_void};

use crate::cxx;

///
#[repr(u8)]
pub enum TypeId {
    Gid = 1,
    Int = 2,
    UInt = 3,
    Flt = 4,
    Byt = 5,
    UByt = 6,
    UShrt = 7,
    Dbl = 8,
    Str = 9,
    WStr = 10,

    Unknown(u8),
}

///
#[repr(C)]
pub struct Field {
    vtable: *mut c_void,
    double_storage: c_double,
    float_storage: c_float,
    int_storage: c_int,
    str_storage: *mut cxx::Str,
    wstr_storage: *mut cxx::WStr,
    gid_storage: c_ulonglong, // XXX: gid union?
    type_id: TypeId,
    _38: [u8; 0x18],
    name: cxx::String,
    _70: [u8; 0x8],
}

assert_eq_size!(Field, [u8; 0x78]);

///
#[repr(C)]
pub struct Record {
    vtable: *mut c_void,
    _08: [u8; 0x10],
    ref_count: u32, // std::atomic?
    fields: cxx::Vector<Field>,
}

// TODO: Figure out real size of `Record` type.
assert_eq_size!(Record, [u8; 0x38]);
