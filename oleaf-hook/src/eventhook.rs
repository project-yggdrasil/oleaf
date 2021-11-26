use std::alloc::{dealloc, Layout};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use detour::static_detour;

#[repr(C)]
union StdStrImpl {
    buf: [c_char; 0x10],
    ptr: *mut c_char,
}

#[repr(C)]
pub struct CxxString {
    ipl: StdStrImpl,
    size: i32,
}

impl CxxString {
    pub fn new<S: ToString>(data: S) -> Self {
        let data = data.to_string();
        if data.len() > 15 {
            Self {
                size: data.len() as i32,
                ipl: StdStrImpl {
                    ptr: Box::into_raw(Box::new(data)) as _,
                },
            }
        } else {
            let mut buf = [0; 0x10];
            buf.copy_from_slice(data.as_bytes());
            Self {
                size: data.len() as i32,
                ipl: StdStrImpl {
                    buf: unsafe { std::mem::transmute(buf) },
                },
            }
        }
    }

    pub fn c_str<'a>(&self) -> &'a CStr {
        unsafe {
            // Real gamers hide unsafety
            if self.size < 16 {
                CStr::from_ptr(self.ipl.buf.as_ptr())
            } else {
                CStr::from_ptr(self.ipl.ptr)
            }
        }
    }
}

impl Drop for CxxString {
    fn drop(&mut self) {
        if self.size > 15 {
            unsafe {
                ptr::drop_in_place(self.ipl.ptr);
                dealloc(self.ipl.ptr as *mut u8, Layout::new::<String>());
            }
        }
    }
}

static_detour! {
    pub static SendEventHook: fn(i64, i64, i64) -> i64;
}

pub type FnSendEvent = fn(i64, i64, i64) -> i64;
pub type FnEventHandlerGetter = fn(i64, i64) -> i64;

pub static mut EVENT_HANDLER_GETTER_PTR: *const () = 0 as *mut ();

pub fn send_event_detour(dispatcher: i64, name: i64, unknown: i64) -> i64 {
    unsafe {
        let testname = CxxString::new("HandleQuestDialog");

        let handler_addr =
            std::mem::transmute::<*const (), FnEventHandlerGetter>(EVENT_HANDLER_GETTER_PTR)(
                dispatcher,
                *std::mem::transmute::<&CxxString, *const *const ()>(&testname) as i64,
            );
        if handler_addr != 0 {
            // We don't need the hook anymore (should do this after all things are found)
            println!("Found HandleQuestDialog at {:x}", handler_addr);

            match SendEventHook.disable() {
                Ok(()) => (),
                Err(err) => println!("{}", err),
            }
        }

        SendEventHook.call(dispatcher, name, unknown)
    }
}
