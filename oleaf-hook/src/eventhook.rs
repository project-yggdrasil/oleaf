use detour::static_detour;

use crate::cxx;

static_detour! {
    pub static SendEventHook: fn(i64, i64, i64) -> i64;
}

pub type FnSendEvent = fn(i64, i64, i64) -> i64;
pub type FnEventHandlerGetter = fn(i64, i64) -> i64;

pub static mut EVENT_HANDLER_GETTER_PTR: *const () = 0 as *mut ();

pub fn send_event_detour(dispatcher: i64, name: i64, unknown: i64) -> i64 {
    unsafe {        
        let handlernames = vec!["HandleActorDialog", "HandleQuestDialog", "Loot", "DisplayLootDialog", "DisplayLootTip", "UpdateSpellSelection", "ZoneTransferRequested", "ZoneAttach"];
        for handlername in handlernames {
            let mut handlername = cxx::String::new(handlername).unwrap();
            let handler_addr = std::mem::transmute::<*const (), FnEventHandlerGetter>(EVENT_HANDLER_GETTER_PTR)(dispatcher, std::mem::transmute::<&cxx::String, *const ()>(&handlername) as i64);
            if handler_addr != 0 {
                // We don't need the hook anymore (should do this after all things are found)
                println!("Found {} at {:x}", handlername.view().to_str().unwrap(), handler_addr);

                match SendEventHook.disable() {
                    Ok(()) => (),
                    Err(err) => println!("{}", err)
                }
            }
        }

        SendEventHook.call(dispatcher, name, unknown)
    }
}
