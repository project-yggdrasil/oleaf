use std::{
    char,
    ffi::NulError,
    os::raw::{c_size_t, c_ushort},
    ptr, slice,
};

#[allow(non_camel_case_types)]
type c_wchar_t = c_ushort;

#[repr(C)]
union Impl {
    // Invariant 1: One byte must always be reserved for trailing null.
    // Invariant 2: No `\0` in the middle of the string supported.
    buf: [c_wchar_t; Self::SSO_LEN],
    ptr: *mut c_wchar_t,
}

// We require this type to be exactly 32 bytes in size.
assert_eq_size!(Impl, [u8; 32]);

impl Impl {
    const SSO_LEN: usize = 0x10;

    // SAFETY: The C++ side MUST NOT modify the resulting string and `len` must be correct.
    pub unsafe fn from_rust<S: Into<Vec<c_wchar_t>>>(value: S) -> Result<Self, NulError> {
        let value: Vec<u16> = value.into();
        let len = value.len();

        if len < Self::SSO_LEN {
            // We have enough capacity to perform short string
            // optimization while preserving the null terminator.
            unsafe {
                let mut buf = [0; Self::SSO_LEN];
                ptr::copy_nonoverlapping(value.as_ptr(), buf.as_mut_ptr(), len);

                Ok(Self { buf })
            }
        } else {
            Ok(Self {
                ptr: Box::<[c_wchar_t]>::into_raw(value.into_boxed_slice()) as *mut c_wchar_t,
            })
        }
    }

    // SAFETY: This function may only be called when  `Impl::from_rust`
    // was used to obtain this object and `len` is an accurate depiction
    // of `wchar_t`-sized quantities in the string.
    pub unsafe fn free_rust(&mut self, len: usize) {
        unsafe {
            let data = slice::from_raw_parts_mut(self.ptr, len);
            let _ = Box::from_raw(data as *mut [c_wchar_t]);
        }
    }
}

/// An ABI-compatible `std::wstring` that is owned by the Rust side.
///
/// This is a null-terminated string that can be created from the Rust
/// side and shared with C++ code.
///
/// Instances of this string are guaranteed to contain no interior
/// null bytes and guarantee that their raw data is null-terminated.
///
/// # Immutability
///
/// When passed to the C++ side, such a string should be treated as
/// immutable and correspondingly not be modified. However as the
/// language has no concept for us to enforce this, we delegate the
/// responsibility to the Rust programmer seeking to call C++ code.
///
/// # Interpretation
///
/// This type should **never** be used to interpret C++-managed
/// `std::string` objects as Rust types. This mandates immediate
/// undefined behavior for the executing program.
///
/// C++ will have its own lifecycle management for string objects
/// and just like we intend C++ code to treat instances of this
/// type immutably, we also want the same vice-versa.
///
/// If a Rust handle on a purely C++-managed string is desired, use
/// the [`Str`] type instead.
#[repr(C)]
pub struct WString {
    ipl: Impl,
    size: c_size_t,
    capacity: c_size_t,
}

impl WString {
    /// Creates a new string from Rust-managed data.
    ///
    /// This function will error if the string contains interior null
    /// bytes ("null characters").
    ///
    /// # Safety
    ///
    /// It is within the caller's responsibility that the resulting string
    /// **remains unmodified** when shared with the C++ side.
    pub unsafe fn new<S: ToString>(data: S) -> Result<Self, NulError> {
        let data: Vec<_> = data.to_string().encode_utf16().collect();
        let size = data.len();

        Ok(Self {
            ipl: unsafe { Impl::from_rust(data)? },
            size,
            capacity: size, // We don't care about allocating extra space
        })
    }

    /// Decodes the bytes stored in a [`CStr`] as UTF-16 and returns the resulting
    /// [`String`] object.
    ///
    /// Invalid code points will be replaced by `U+FFFD REPLACEMENT CHARACTER` (�)
    /// in the resulting string.
    pub fn decode_utf16(&self) -> String {
        unsafe {
            if self.size < Impl::SSO_LEN {
                decode_escaped_utf16(&self.ipl.buf[..self.size])
            } else {
                let utf16 = slice::from_raw_parts(self.ipl.ptr, self.size);
                decode_escaped_utf16(utf16)
            }
        }
    }
}

// Turns this `WString` into an empty string to prevent
// memory-unsafe code from working by accident. Inline
// to prevent LLVM from optimizing it away in debug builds.
impl Drop for WString {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            if self.size < Impl::SSO_LEN {
                *self.ipl.buf.get_unchecked_mut(0) = 0;
            } else {
                ptr::write(self.ipl.ptr, 0);
                // SAFETY: The string was allocated by Rust code without SSO.
                self.ipl.free_rust(self.size)
            }
        }
    }
}

/// An ABI-compatible `std::wstring` that is borrowed from the C++ side.
///
/// This represents a null-terminated string created and managed by C++
/// code to which an immutable view on the Rust side is granted.
///
/// No assumptions are made on the underlying data and caution must be
/// applied for the special kind of programmers who use `std::string`
/// as a storage for binary data.
///
/// # Safety
///
/// The user must ensure that their handle to a [`Str`] instance does not
/// outlive the corresponding object on the C++ side.
#[repr(C)]
pub struct WStr {
    ipl: Impl,
    size: c_size_t,
    capacity: c_size_t,
}

impl WStr {
    /// Decodes the bytes stored in a [`CStr`] as UTF-16 and returns the resulting
    /// [`String`] object.
    ///
    /// Invalid code points will be replaced by `U+FFFD REPLACEMENT CHARACTER` (�)
    /// in the resulting string.
    ///
    /// # Safety
    ///
    /// The data directly originates from C++, no validation is done to ensure this
    /// is actually valid UTF-16 under the hood. This may not actually be valid
    /// string data.
    pub unsafe fn decode_utf16(&self) -> String {
        unsafe {
            if self.size < Impl::SSO_LEN {
                decode_escaped_utf16(&self.ipl.buf[..self.size])
            } else {
                let utf16 = slice::from_raw_parts(self.ipl.ptr, self.size);
                decode_escaped_utf16(utf16)
            }
        }
    }
}

fn decode_escaped_utf16(utf16: &[u16]) -> String {
    char::decode_utf16(utf16.iter().copied())
        .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
        .collect()
}
