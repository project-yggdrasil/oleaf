use std::{lazy::SyncOnceCell, os::raw::c_void};

use detour::static_detour;

use crate::cxx;

// Not part of the public API. Used by generated code.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static INIT_EVENT_DETOURS: [unsafe fn(*mut c_void)] = [..];

// Not part of the public API. Used by generated code.
#[doc(hidden)]
#[linkme::distributed_slice]
pub static UNHOOK_EVENT_DETOURS: [unsafe fn()] = [..];

static_detour! {
    /// The detour for installing custom event handlers.
    ///
    /// The crate user should install [`send_event_detour`] to this hook.
    pub static SendEventHook: unsafe extern "fastcall" fn(
        /* dispatcher: */ *mut c_void,
        /* name: */ *mut cxx::Str,
        /* unknown: */ *mut c_void
    ) -> *mut c_void;
}

/// The function signature for [`SendEventHook`] detours.
pub type FnSendEvent =
    unsafe extern "fastcall" fn(*mut c_void, *mut cxx::Str, *mut c_void) -> *mut c_void;

/// The function signature for a dispatcher's event handler getter.
pub type FnGetEventHandler = extern "fastcall" fn(*mut c_void, *mut cxx::String) -> *mut c_void;

static EVENT_HANDLER_GETTER: SyncOnceCell<FnGetEventHandler> = SyncOnceCell::new();

/// Initializes the global event handler getter to the given functions.
///
/// # Panics
///
/// Panics if the callback has already been initialized previously.
pub fn initialize_event_handler_getter(func: FnGetEventHandler) {
    EVENT_HANDLER_GETTER.set(func).unwrap_or_else(|eg| {
        panic!(
            "Failed to initialize event handler getter to {:#p}!",
            eg as *const c_void
        )
    });
}

/// Attempts to find an event handler by name, returning a pointer to its
/// callback function on success.
pub fn find_event_by_name(dispatcher: *mut c_void, name: &mut cxx::String) -> Option<*mut c_void> {
    let handler = EVENT_HANDLER_GETTER
        .get()
        .expect("Event handler getter was not yet initialized!");

    let ptr = handler(dispatcher, name as *mut cxx::String);
    if !ptr.is_null() {
        Some(ptr)
    } else {
        None
    }
}

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
#[allow(clippy::not_unsafe_ptr_arg_deref, unsafe_op_in_unsafe_fn)]
#[inline(never)]
pub fn send_event_detour(
    dispatcher: *mut c_void,
    name: *mut cxx::Str,
    unk: *mut c_void,
) -> *mut c_void {
    // Get a handle to the dispatcher object and call all event detour installers.
    for ptr in INIT_EVENT_DETOURS {
        unsafe { ptr(dispatcher) }
    }

    // Call the original C++ function.
    unsafe { SendEventHook.call(dispatcher, name, unk) }
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
