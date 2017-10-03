#![feature(proc_macro)]
#![recursion_limit="128"]

extern crate inflector;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate synom;

mod actor;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn actor_impl(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let source = item.to_string();
    let impl_ast = syn::parse_item(&source).unwrap();
    let tokens_out = actor::codegen_actor_impl(impl_ast);
    tokens_out.parse().unwrap()
}

#[proc_macro_attribute]
pub fn actor_api(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let source = item.to_string();
    let impl_ast = syn::parse_item(&source).unwrap();
    let tokens_out = actor::codegen_actor_api(impl_ast);
    tokens_out.parse().unwrap()
}

#[proc_macro_attribute]
pub fn on_start(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let source = item.to_string();
    let impl_ast = syn::parse_item(&source).unwrap();
    let tokens_out = actor::codegen_actor_on_start(impl_ast);
    tokens_out.parse().unwrap()
}

#[proc_macro_attribute]
pub fn on_stop(_attribute: TokenStream, item: TokenStream) -> TokenStream {
    let source = item.to_string();
    let impl_ast = syn::parse_item(&source).unwrap();
    let tokens_out = actor::codegen_actor_on_stop(impl_ast);
    tokens_out.parse().unwrap()
}
