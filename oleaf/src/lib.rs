use windows::Win32::Foundation::{BOOL, HINSTANCE};

#[cfg(not(all(target_arch = "x86_64", target_os = "windows")))]
compile_error!("Only x64 builds for Windows are supported!");

/// Entrypoint to the oleaf application.
#[allow(clippy::missing_safety_doc, non_snake_case)]
#[no_mangle]
pub unsafe extern "system" fn DllMain(
    _module: HINSTANCE,
    _call_reason: u32,
    _reserved: *const (),
) -> BOOL {
    todo!()
}
