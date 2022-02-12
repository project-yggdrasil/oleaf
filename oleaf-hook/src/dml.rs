//! ABI-compatible types depicting relevant primitives of the DML system.

use std::os::raw::*;

use crate::cxx;

/// A unique ID that indicates the type of a DML [`Field`].
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

/// A DML field that is part of a [`Record`].
///
/// This type is borrowed from C++ client code for introspection on the
/// Rust side and should never be constructed manually.
#[repr(C)]
pub struct Field {
    vtable: *mut c_void,
    double_storage: c_double,
    float_storage: c_float,
    int_storage: c_int,
    str_storage: *mut cxx::Str,
    wstr_storage: *mut cxx::WStr,
    gid_storage: c_ulonglong,
    type_id: TypeId,
    _38: [u8; 0x18],
    name: cxx::Str,
    _70: [u8; 0x8],
}

/// Representation of DML field values for introspection in Rust code.
pub enum FieldValue<'dml> {
    Byt(c_char),
    UByt(c_uchar),
    UShrt(c_ushort),
    Int(c_int),
    UInt(c_uint),
    Gid(c_ulonglong),
    Flt(c_float),
    Dbl(c_double),
    Str(&'dml cxx::Str),
    WStr(&'dml cxx::WStr),
}

impl Field {
    /// Gets the name of the field.
    ///
    /// # Safety
    ///
    /// The lifetime of `self` and thus also the inferred lifetime of the
    /// return value may not be representative of the real lifetime of
    /// the data.
    ///
    /// The caller is responsible for ensuring the availability of the
    /// requested data.
    pub unsafe fn name(&self) -> &cxx::Str {
        &self.name
    }

    /// Gets the value of this field if its type can be determined.
    ///
    /// # Safety
    ///
    /// The lifetime of `self` and thus also the inferred lifetime of the
    /// [`FieldValue`] may not be representative of the real lifetime of
    /// the data.
    ///
    /// The caller is responsible for ensuring the availability of the
    /// requested data.
    pub unsafe fn value(&self) -> Option<FieldValue<'_>> {
        match self.type_id {
            TypeId::Gid => Some(FieldValue::Gid(self.gid_storage)),
            TypeId::Int => Some(FieldValue::Int(self.int_storage)),
            TypeId::UInt => Some(FieldValue::UInt(self.int_storage as c_uint)),
            TypeId::Flt => Some(FieldValue::Flt(self.float_storage)),
            TypeId::Byt => Some(FieldValue::Byt(self.int_storage as c_char)),
            TypeId::UByt => Some(FieldValue::UByt(self.int_storage as c_uchar)),
            TypeId::UShrt => Some(FieldValue::UShrt(self.int_storage as c_ushort)),
            TypeId::Dbl => Some(FieldValue::Dbl(self.double_storage)),
            TypeId::Str => Some(FieldValue::Str(unsafe { &*self.str_storage })),
            TypeId::WStr => Some(FieldValue::WStr(unsafe { &*self.wstr_storage })),

            _ => None,
        }
    }
}

assert_eq_size!(Field, [u8; 0x78]);

/// A DML record that groups together several DML [`Field`]s holding data.
///
/// Records are the bodies of DML messages and the primary format of
/// structured data in the game. We will exfiltrate most information
/// from records.
#[repr(C)]
pub struct Record {
    vtable: *mut c_void,
    _08: [u8; 0x10],
    ref_count: u32,
    fields: cxx::Vector<Field>,
}

impl Record {
    /// Gets a slice holding all the [`Field`]s in the record.
    ///
    /// # Safety
    ///
    /// The lifetime of `self` and thus also the inferred lifetime of the
    /// return value may not be representative of the real lifetime of
    /// the data.
    ///
    /// The caller is responsible for ensuring the availability of the
    /// requested data.
    pub unsafe fn fields(&self) -> &[Field] {
        unsafe { self.fields.as_slice() }
    }
}

assert_eq_size!(Record, [u8; 0x38]);
