use std::{
    ffi::{CStr, CString, NulError},
    os::raw::{c_char, c_size_t},
    ptr,
};

#[repr(C)]
pub union Impl {
    // Invariant 1: One byte must always be reserved for trailing null.
    // Invariant 2: No `\0` in the middle of the string supported.
    buf: [c_char; Self::SSO_LEN],
    ptr: *mut c_char,
}

// We require this type to be exactly 16 bytes in size.
assert_eq_size!(Impl, [u8; 16]);

impl Impl {
    const SSO_LEN: usize = 0x10;

    // SAFETY: The C++ side MUST NOT modify the resulting string and `len` must be correct.
    pub unsafe fn from_rust<S: Into<Vec<u8>>>(value: S, len: usize) -> Result<Self, NulError> {
        let cstr = CString::new(value)?;

        if len < Self::SSO_LEN {
            // We have enough capacity to perform short string
            // optimization while preserving the null terminator.
            unsafe {
                let mut buf = [0; Self::SSO_LEN];
                ptr::copy_nonoverlapping(
                    cstr.as_bytes().as_ptr() as *const i8,
                    buf.as_mut_ptr(),
                    len,
                );

                Ok(Self { buf })
            }
        } else {
            Ok(Self {
                ptr: cstr.into_raw(),
            })
        }
    }

    // SAFETY: This function may only be called when no SSO was
    // employed, i.e. the input string was shorter than 16 chars,
    // and when `Impl::from_rust` was used to obtain this object.
    pub unsafe fn free_rust(&mut self) {
        unsafe {
            let _ = CString::from_raw(self.ptr);
        }
    }
}

/// An ABI-compatible `std::string` that is owned by the Rust side.
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
pub struct String {
    ipl: Impl,
    size: c_size_t,
}

impl String {
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
        let data = data.to_string();
        let size = data.len();

        Ok(Self {
            ipl: unsafe { Impl::from_rust(data, size)? },
            size,
        })
    }

    /// Gets a [`CStr`] view to the underlying string data.
    ///
    /// The underlying data is guaranteed to be valid Rust string data
    /// when all safety rules of this type were respected.
    pub unsafe fn view(&mut self) -> &CStr {
        unsafe {
            if self.size < Impl::SSO_LEN {
                CStr::from_ptr(self.ipl.buf.as_mut_ptr())
            } else {
                CStr::from_ptr(self.ipl.ptr)
            }
        }
    }
}

impl Drop for String {
    fn drop(&mut self) {
        if self.size >= Impl::SSO_LEN {
            // SAFETY: The string was created in Rust and is not SSO-optimized.
            unsafe { self.ipl.free_rust() }
        }
    }
}

/// An ABI-compatible `std::string` that is borrowed from the C++ side.
///
/// This represents a null-terminated string created and managed by C++
/// code to which an immutable view on the Rust side was obtained.
///
/// No assumptions are made on the underlying data and caution must be
/// applied for the special kind of programmers who use `std::string`
/// as a storage for binary data.
#[repr(C)]
pub struct Str {
    ipl: Impl,
    size: c_size_t,
}

impl Str {
    /// Gets a [`CStr`] view to the underlying string data.
    ///
    /// # Safety
    ///
    /// This function is unsafe for the same reasons as [`CStr::from_ptr`].
    /// The underlying pointers and pointed-to data originate purely
    /// from C++ code without any form of scrutiny.
    pub unsafe fn view(&mut self) -> &CStr {
        unsafe {
            if self.size < Impl::SSO_LEN {
                CStr::from_ptr(self.ipl.buf.as_mut_ptr())
            } else {
                CStr::from_ptr(self.ipl.ptr)
            }
        }
    }
}
