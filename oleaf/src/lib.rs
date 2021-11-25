use std::error::Error;

use windows::Win32::Foundation::{BOOL, HINSTANCE};
use windows::Win32::System::SystemServices::{DLL_PROCESS_ATTACH};

use oleaf_hook::module::Module;
use oleaf_hook::eventhook::*;

#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("Only x64 builds for Windows are supported!");

unsafe fn main() -> Result<(), Box<dyn Error>> {
    let cur_mod = Module::pe().ok_or("Failed to get the current module")?;
    let send_event_target: FnSendEvent = std::mem::transmute(cur_mod.find_signature("40 ?? 56 57 41 ?? 41 ?? 41 ?? 41 ?? 48 81 ?? ?? ?? ?? ?? ?? c7 ?? ?? ?? ?? ?? ?? ?? ?? 89 ?? ?? ?? ?? ?? ?? 48 8b ?? ?? ?? ?? ?? 48 33 ?? ?? 89 ?? ?? ?? ?? ?? ?? 4d 8b ?? ?? 89")?);

    EVENT_HANDLER_GETTER_PTR = std::mem::transmute(cur_mod.find_signature("41 56 48 83 EC ?? 48 C7 44 24 20 FE FF FF FF 48 89 5C 24 ?? 48 89 6C 24 ?? 48 89 74 24 ?? 48 89 7C 24 ?? 48 8B FA 4C 8B C9")?);
    println!("EventHandler getter found at: {:x}", EVENT_HANDLER_GETTER_PTR as i64);

    SendEventHook
        .initialize(send_event_target, send_event_detour)?
        .enable()?;

    println!("Hooked SendEvent\n");
    
    Ok(())
}

/// Entrypoint to the oleaf application.
#[allow(clippy::missing_safety_doc, non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    call_reason: u32,
    _reserved: *const (),
) -> BOOL {
    BOOL::from(
        if call_reason == DLL_PROCESS_ATTACH {
            main().is_ok()
        } else {
            true
        }
    )
}
