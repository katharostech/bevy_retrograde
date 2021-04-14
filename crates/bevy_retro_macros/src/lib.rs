use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{parse_macro_input, spanned::Spanned, token::Pub, ItemFn, VisPublic, Visibility};

/// Attribute macro that should be added to bevy retro `main` functions to add the extra wrappers
/// needed to support web.
#[proc_macro_attribute]
pub fn main(_args: TokenStream, input: TokenStream) -> TokenStream {
    let function = parse_macro_input!(input as ItemFn);
    impl_language_adapter(function).into()
}

fn impl_language_adapter(mut function: ItemFn) -> TokenStream2 {
    if function.sig.ident != format_ident!("main") {
        return quote! {
            compile_error!("bevy_retro `main` macro must only be added to the main function");
        };
    }
    function.sig.ident = format_ident!("__bevy_retro_start");
    function.vis = Visibility::Public(VisPublic {
        pub_token: Pub(function.vis.span()),
    });

    quote! {
        fn main() {
            #[cfg(not(target_arch = "wasm32"))]
            __bevy_retro_start();
        }

        #[cfg_attr(target_arch = "wasm32", ::bevy_retro::__macro_deps::wasm_bindgen::prelude::wasm_bindgen(js_name = "start"))]
        #function
    }
}
