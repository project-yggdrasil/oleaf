use std::{error::Error, ffi::c_void, ptr};

use oleaf_hook::event;
use windows::Win32::{
    Foundation::{BOOL, HINSTANCE},
    System::{
        Console,
        LibraryLoader::DisableThreadLibraryCalls,
        SystemServices::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH},
        Threading::{CreateThread, THREAD_CREATE_RUN_IMMEDIATELY},
    },
};

#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("Only x64 builds for Windows are supported!");

#[oleaf_hook::event("HandleQuestDialog")]
fn handle_quest_dialog(this: *mut c_void, dml: *mut oleaf_hook::dml::Record) {
    println!("Works");

    call_original!(this, dml)
}

const SEND_EVENT_SIG: &str = "40 ?? 56 57 41 ?? 41 ?? 41 ?? 41 ?? 48 81 ?? ?? ?? ?? ?? ?? c7 ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 48 8b ?? ?? ?? ?? ?? 48 33 ?? ?? 89 ?? ?? ?? ?? ?? ?? 4d 8b ?? ?? 89";
const EVENT_HANDLER_GETTER_SIG: &str = "41 56 48 83 EC ?? 48 C7 44 24 20 FE FF FF FF 48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 48 8B FA 4C 8B C9";

unsafe fn initialize_detours() -> Result<(), Box<dyn Error>> {
    let cur_mod = oleaf_hook::Module::pe().ok_or("Failed to find module")?;

    let send_event_target: event::FnSendEvent =
        std::mem::transmute(cur_mod.find_signature(SEND_EVENT_SIG)?);
    let event_handler_getter: event::FnGetEventHandler =
        std::mem::transmute(cur_mod.find_signature(EVENT_HANDLER_GETTER_SIG)?);

    println!(
        "EventHandler getter found at: {:x}",
        event_handler_getter as usize
    );
    event::initialize_event_handler_getter(event_handler_getter);

    event::SendEventHook
        .initialize(send_event_target, event::send_event_detour)?
        .enable()?;

    println!("Hooked SendEvent\n");

    Ok(())
}

#[inline(never)]
unsafe extern "system" fn bootstrap_oleaf(_: *mut c_void) -> u32 {
    initialize_detours().is_err() as u32
}

unsafe fn main(module: HINSTANCE, call_reason: u32) -> Result<(), Box<dyn Error>> {
    match call_reason {
        DLL_PROCESS_ATTACH => {
            DisableThreadLibraryCalls(module).ok()?;
            Console::AllocConsole().ok()?;

            // Bootstrap the functionality in a separate thread.
            CreateThread(
                ptr::null(),
                0,
                Some(bootstrap_oleaf),
                ptr::null(),
                THREAD_CREATE_RUN_IMMEDIATELY,
                ptr::null_mut(),
            );

            Ok(())
        }
        DLL_PROCESS_DETACH => {
            Console::FreeConsole().ok()?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Entrypoint to the oleaf application.
#[allow(clippy::missing_safety_doc, non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    module: HINSTANCE,
    call_reason: u32,
    _reserved: *const (),
) -> BOOL {
    main(module, call_reason).is_ok().into()
}
