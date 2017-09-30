#![feature(proc_macro)]

extern crate inflector;
extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;
#[macro_use]
extern crate synom;

use proc_macro::TokenStream;
use syn::parse::{block, ident, ty};
use inflector::Inflector;


#[proc_macro]
pub fn actor(input: TokenStream) -> TokenStream {
    let source = input.to_string();
    let dsl_ast = parse_actor(&source).expect("Failed to parse actor DSL");
    let tokens = codegen_actor(&dsl_ast);
    println!("{}", tokens);
    tokens.parse().unwrap()
}

#[derive(Debug)]
struct ActorMessageField {
    name: syn::Ident,
    tipe: syn::Ty,
}

impl ActorMessageField {
    pub fn new(name: syn::Ident, tipe: syn::Ty) -> Self {
        Self {
            name: name,
            tipe: tipe,
        }
    }
}

#[derive(Debug)]
struct ActorMessage {
    name: syn::Ident,
    fields: Vec<ActorMessageField>,
    body: syn::Block,
}

impl ActorMessage {
    pub fn new(name: syn::Ident, fields: Vec<ActorMessageField>, body: syn::Block) -> Self {
        Self {
            name: name,
            fields: fields,
            body: body,
        }
    }
}

#[derive(Debug)]
struct ActorDefinition {
    name: syn::Ident,
    messages: Vec<ActorMessage>,
}

impl ActorDefinition {
    pub fn new(name: syn::Ident, messages: Vec<ActorMessage>) -> Self {
        Self {
            name: name,
            messages: messages,
        }
    }

    pub fn message_name(&self) -> syn::Ident {
        syn::Ident::from(format!("{}Message", self.name.as_ref()))
    }
}

named! { parse_actor -> ActorDefinition,
    do_parse!(
        name: ident >> punct!(",") >>
        body: many0!(parse_message) >>

        (ActorDefinition::new(name, body))
    )
}

named! { parse_message -> ActorMessage,
    do_parse! (
        punct!("message") >>
        name: ident >>
        args: delimited!(punct!("("), terminated_list!(punct!(","), parse_message_field), punct!(")")) >>
        body: block >>

        (ActorMessage::new(name, args, body))
    )
}

named! { parse_message_field -> ActorMessageField,
    do_parse! (
        name: ident >>
        punct!(":") >>
        tipe: ty >>

        (ActorMessageField::new(name, tipe))
    )
}

fn codegen_actor(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let message_enum = codegen_message_enum(dsl_ast);
    let actor_struct = codegen_actor_struct(dsl_ast);
    let actor_impl = codegen_actor_impl(dsl_ast);
    quote!{
        #message_enum
        #actor_struct
        #actor_impl
    }
}

fn codegen_message_enum(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let enum_name = dsl_ast.message_name();
    let message_variants = dsl_ast.messages.iter().map(codegen_message_variant);
    quote! {
        enum #enum_name {
            #(#message_variants,)*
        }
    }
}

fn codegen_message_variant(message_ast: &ActorMessage) -> quote::Tokens {
    let tipes = message_ast.fields.iter().map(|field| &field.tipe);
    let name = &message_ast.name;
    quote! {
        #name(#(#tipes,)*)
    }
}

fn codegen_actor_struct(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = &dsl_ast.name;
    quote! {
        struct #name {
        }
    }
}

fn codegen_actor_impl(dsl_ast: &ActorDefinition) -> quote::Tokens {
    let name = &dsl_ast.name;
    let message_name = dsl_ast.message_name();
    let mod_name = syn::Ident::from(format!("_impl_actor_{}", name.as_ref().to_snake_case()));
    let message_handlers = dsl_ast
        .messages
        .iter()
        .map(|message| codegen_message_handler(&message_name, message));
    quote!{
        mod #mod_name {
            use akio::Actor;
            impl Actor for #name {
                type Message = #message_name;

                fn handle_message(&mut self, context: &mut ActorContext, message: Self::Message) {
                    match message {
                        #(#message_handlers,)*
                    }
                }
            }
        }
    }
}

fn codegen_message_handler(message_enum_name: &syn::Ident,
                           message_ast: &ActorMessage)
                           -> quote::Tokens {
    let message_name = &message_ast.name;
    let message_field_names = message_ast.fields.iter().map(|field| &field.name);
    let message_body = &message_ast.body;
    quote! {
        #message_enum_name::#message_name(#(#message_field_names,)*) => #message_body
    }
}
