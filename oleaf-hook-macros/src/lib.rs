//! Procedural macros for use with [`oleaf-hook`].
//!
//! These macros are re-exported by the [`oleaf-hook`] crate itself,
//! there is no need to explicitly add this crate to dependencies.
//!
//! [`oleaf-hook`]: ../oleaf_hook/

#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

#[macro_use]
extern crate quote;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream as TokenStream2;
use syn::{
    parse_macro_input, punctuated::Punctuated, token::Comma, BareFnArg, Error, FnArg, ItemFn,
    LitStr, Result,
};

/// Declares a new event handler detour for any of the client events.
///
/// This attribute takes a single string literal which names the event
/// and decorates the detour function:
///
/// ```ignore
/// # use oleaf_hook_macros::event;
/// #[event("HandleActorDialog")]
/// unsafe extern "fastcall" fn actor_dialog_handler(/* arguments */) {
///     // ...
///     call_original!(/* arguments */)
/// }
/// ```
///
/// Within the handler function, a special `call_original` macro will
/// be available for forwarding any arguments to the detoured function.
///
/// Note that the `detour`, `linkme` and `oleaf-hook` crates are required
/// as direct dependencies of any crate this macro is used in.
#[proc_macro_attribute]
pub fn event(attr: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let event = parse_macro_input!(attr as LitStr);
    let func = parse_macro_input!(item as ItemFn);

    expand(func, event)
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn into_bare_args(args: &Punctuated<FnArg, Comma>) -> Punctuated<BareFnArg, Comma> {
    args.iter()
        .map(|arg| {
            if let FnArg::Typed(pat_type) = arg {
                BareFnArg {
                    attrs: pat_type.attrs.clone(),
                    name: None,
                    ty: (*pat_type.ty).clone(),
                }
            } else {
                todo!()
            }
        })
        .collect()
}

fn expand(func: ItemFn, event: LitStr) -> Result<TokenStream2> {
    let attrs = &func.attrs;
    let vis = &func.vis;
    let sig = &func.sig;
    let sig_ty = syn::Type::BareFn(syn::TypeBareFn {
        lifetimes: None,
        unsafety: sig.unsafety,
        abi: sig.abi.clone(),
        fn_token: sig.fn_token,
        paren_token: sig.paren_token,
        inputs: into_bare_args(&sig.inputs),
        variadic: sig.variadic.clone(),
        output: sig.output.clone(),
    });
    let ident = &sig.ident;
    let block = &func.block;

    if let Some(asyncness) = &sig.asyncness {
        return Err(Error::new_spanned(
            asyncness,
            "function must not be declared as async fn",
        ));
    }

    let detour_ident = format_ident!("__{}_OLEAF_ORIGINAL", ident);
    Ok(quote! {
        use ::detour::static_detour;
        static_detour! {
            static #detour_ident: #sig_ty;
        }

        const _: () = {
            type __EventDetourFn = #sig_ty;

            #[allow(unsafe_op_in_unsafe_fn)]
            #[::linkme::distributed_slice(::oleaf_hook::event::INIT_EVENT_DETOURS)]
            unsafe fn __install_detour(dispatcher: *mut ::std::os::raw::c_void) {
                use ::core::{option::Option, result::Result};

                // Opt out if the event handler for this function is already installed.
                if #detour_ident.is_enabled() {
                    return;
                }

                // See if we can query the handler for our event...
                let mut event = ::oleaf_hook::cxx::String::new(#event)
                    .expect(::core::concat!("Got invalid handler name: ", #event, "!"));
                if let Option::Some(ptr) = ::oleaf_hook::event::find_event_by_name(dispatcher, &mut event) {
                    ::oleaf_hook::paging::with_read_write_page(ptr, 0x10, || {
                        // ...and detour it.
                        #detour_ident.initialize(
                            ::core::mem::transmute::<_, __EventDetourFn>(ptr),
                            #ident,
                        )
                        .unwrap_or_else(|e| {
                            ::core::panic!(::core::concat!("Failed to install detour for ", #event, ": {}!"), e);
                        });
                        #detour_ident.enable()
                            .expect(::core::concat!("Failed to enable detour for ", #event));
                    })
                    .expect("Failed to alter page table permissions to read/write!");
                }
            }

            #[allow(unsafe_op_in_unsafe_fn)]
            #[::linkme::distributed_slice(::oleaf_hook::event::UNHOOK_EVENT_DETOURS)]
            unsafe fn __uninstall_detour() {
                let _ = #detour_ident.disable();
            }
        };

        #[allow(unused_braces, unused_macros)]
        #(#attrs)*
        #vis #sig {
            // Injected into scope for use by the function author.
            macro_rules! call_original {
                ($($tt:tt)*) => {
                    #detour_ident.call($($tt)*)
                };
            }

            {
                #block
            }
        }
    })
}
