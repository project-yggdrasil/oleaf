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
    _31: [u8; 0x47],
}

assert_eq_size!(Field, [u8; 0x78]);

///
#[repr(C)]
pub struct Record {
    _00: [u8; 0x20],
    fields: cxx::Vector<Field>,
}

// TODO: Figure out real size of `Record` type.
assert_eq_size!(Record, [u8; 0x38]);
