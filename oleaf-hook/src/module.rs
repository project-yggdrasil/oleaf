use std::{
    fmt, io,
    mem::{self, MaybeUninit},
    slice,
};

use windows::Win32::System::{
    LibraryLoader::GetModuleHandleA,
    ProcessStatus::{K32GetModuleInformation, MODULEINFO},
    Threading::GetCurrentProcess,
};

/// Holds information on the module that is being hooked.
pub struct Module<'a> {
    memory: &'a [u8],
}

impl<'a> Module<'a> {
    /// Gets a handle to this module from which we're operating.
    pub fn pe() -> Option<Self> {
        let mut module_info = MaybeUninit::<MODULEINFO>::uninit();
        if unsafe {
            !K32GetModuleInformation(
                GetCurrentProcess(),
                GetModuleHandleA(None),
                module_info.as_mut_ptr(),
                mem::size_of::<MODULEINFO>() as u32,
            )
        }
        .as_bool()
        {
            None
        } else {
            unsafe {
                let module = module_info.assume_init();
                let memory = slice::from_raw_parts(
                    module.lpBaseOfDll as *const u8,
                    module.SizeOfImage as usize,
                );

                Some(Self { memory })
            }
        }
    }

    /// Gets the base address of this module in memory.
    pub fn base(&self) -> usize {
        self.memory.as_ptr() as usize
    }

    /// Gets the size of this module in memory, measured in bytes.
    pub fn size(&self) -> usize {
        self.memory.len()
    }

    /// Finds the first address in this module that matches the signature
    /// `pattern` and returns a pointer to the byte at that address.
    ///
    /// `pattern` is a string where every byte is represented as hexadecimal
    /// characters separated by whitespace. For wildcard matches where a byte
    /// is not a constant `??` shall be used.
    ///
    /// Example: `AB 01 32 ?? 48`
    pub fn find_signature(&self, pattern: &str) -> io::Result<*const u8> {
        let pattern_elements: Vec<_> = pattern
            .split(' ')
            .map(|e| u8::from_str_radix(e, 16).ok())
            .collect();

        self.memory
            .windows(pattern_elements.len())
            .find(|window| {
                pattern_elements
                    .iter()
                    .zip(window.iter())
                    .all(|(pattern_byte, window_byte)| {
                        pattern_byte.map_or(true, |pred| pred == *window_byte)
                    })
            })
            .map(|window| window.as_ptr())
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::UnexpectedEof,
                    "failed to find signature pattern in PE memory",
                )
            })
    }
}

impl<'a> fmt::Debug for Module<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "module at {:#x} - {:#x}",
            self.base(),
            self.base() + self.size()
        )
    }
}
