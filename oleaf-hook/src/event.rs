use std::{ffi::NulError, os::raw::c_void};

use detour::static_detour;

use crate::cxx;

// Not part of the public API. Used by generated code.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static INIT_EVENT_DETOURS: [unsafe fn(&mut Dispatcher)] = [..];

// Not part of the public API. Used by generated code.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static UNHOOK_EVENT_DETOURS: [unsafe fn()] = [..];

/// Representation of KingsIsle's event dispatcher type.
pub struct Dispatcher {}

impl Dispatcher {
    /// Gets an event handler by its string name.
    ///
    /// # Safety
    ///
    /// The lifetime of the result may not be representative of the real
    /// lifetime of the data.
    ///
    /// It is within the caller's responsibility to ensure the availability
    /// at access.
    pub unsafe fn get_handler_by_name(&mut self, str: &str) -> Result<Option<&Handler>, NulError> {
        // Build the arguments for the call to the C++ function.
        let _self_ptr = self as *mut Self;
        let _cxx_string = unsafe { cxx::String::new(str)? };

        todo!("call to the handler getter by vtable and wrap the result")
    }
}

/// Representation of KingsIsle's event handler type.
#[repr(C)]
pub struct Handler {}

static_detour! {
    /// The detour for installing custom event handlers.
    ///
    /// The crate user should install [`send_event_detour`] to this hook.
    pub static SendEventHook: unsafe extern "fastcall" fn(
        /* dispatcher: */ *mut Dispatcher,
        /* name: */ *mut cxx::Str,
        /* unknown: */ *mut c_void
    ) -> *mut c_void;
}

/// The function signature for [`SendEventHook`] detours.
pub type FnSendEvent =
    unsafe extern "fastcall" fn(*mut Dispatcher, *mut cxx::Str, *mut c_void) -> *mut c_void;

/// A detour for [`SendEventHook`] that is used to install custom event
/// handlers for data exfiltration.
///
/// The initialization of the hook should be performed by the crate user.
///
/// This will also set up all the detours that were defined using the
/// [`oleaf_hook::event`] macro.
///
/// Use [`unhook_all`] to uninstall all the detours.
///
/// # Safety
///
/// C++ land. Do not try to call this yourself.
#[allow(unsafe_op_in_unsafe_fn)]
#[inline(never)]
pub unsafe extern "fastcall" fn send_event_detour(
    dispatcher: *mut Dispatcher,
    name: *mut cxx::Str,
    unk: *mut c_void,
) -> *mut c_void {
    // Get a handle to the dispatcher object and call all event detour installers.
    let dispatcher = &mut *dispatcher;
    for ptr in INIT_EVENT_DETOURS {
        ptr(dispatcher);
    }

    // Call the original C++ function.
    SendEventHook.call(dispatcher, name, unk)
}

/// Unhooks all event handler detours that were set up by [`send_event_detour`].
///
/// # Safety
///
/// C++ land. Use at your own risk.
#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn unhook_all() {
    for ptr in UNHOOK_EVENT_DETOURS {
        ptr();
    }
    let _ = SendEventHook.disable();
}
