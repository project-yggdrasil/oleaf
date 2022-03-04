//! Utilities for handling page mapping of the game.

use std::os::raw::c_void;

use windows::{
    core::Error,
    Win32::System::Memory::{VirtualProtect, PAGE_EXECUTE_READWRITE},
};

/// Executes a given closure `F`, with page mapping set to read/write for the duration
/// of the call.
///
/// `addr` is the starting address for the page table change, whereas `size` is the
/// contiguous size in memory for which the setting applies.
///
/// # Safety
///
/// `addr` and `size` are unchecked.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn with_read_write_page<F, T>(addr: *const c_void, size: usize, f: F) -> Result<T, Error>
where
    F: FnOnce() -> T,
{
    let mut protect = Default::default();

    VirtualProtect(addr, size, PAGE_EXECUTE_READWRITE, &mut protect as *mut _).ok()?;
    let res = f();
    VirtualProtect(addr, size, protect, &mut protect as *mut _).ok()?;

    Ok(res)
}
